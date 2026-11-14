use logickiosk_lib::agents::judge::AstJudgeAgent;
use logickiosk_lib::agents::socratic::SocraticTutorAgent;
use logickiosk_lib::agents::{AstInspectionReport, LevelRuleConfig, TestCaseSpec};
use logickiosk_lib::db::repository::DatabaseRepository;
use logickiosk_lib::ipc::commands::{core_get_curriculum, core_submit_code};
use logickiosk_lib::sandbox::runner::SandboxRunner;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_ipc_sqlite_in_memory_atomic_progression_all_5_levels() {
    let repo = DatabaseRepository::connect_and_migrate("sqlite::memory:")
        .await
        .expect("In-memory SQLite migration should succeed");

    let initial = core_get_curriculum(&repo).await.unwrap();
    assert_eq!(initial.modules.len(), 1);
    assert_eq!(initial.levels.len(), 5);
    assert_eq!(initial.levels[0].status, "UNLOCKED");
    assert_eq!(initial.levels[1].status, "LOCKED");

    // Hard gatekeeping: tentar submeter na fase 1.2 ainda bloqueada deve falhar
    let locked_err = core_submit_code(&repo, "lvl_02_condicionais", "def f(x): return x", Some(10), None, None)
        .await
        .expect_err("Locked level submission must fail");
    assert!(locked_err.contains("bloqueada"));

    // Fase 1.1: Hardcode deve ser rejeitado
    let hardcode_res = core_submit_code(
        &repo,
        "lvl_01_variaveis",
        "def calcular_energia_liquida(a, b):\n    return 76.0\n",
        Some(15),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(hardcode_res.verdict, "AST_REJECTED");
    assert!(hardcode_res.ast_report.suspected_hardcode);
    assert!(hardcode_res.socratic_hint.is_some());

    // Fase 1.1: Solução válida
    let pass_1 = core_submit_code(
        &repo,
        "lvl_01_variaveis",
        "def calcular_energia_liquida(geracao_kwh, consumo_kwh):\n    saldo = (geracao_kwh - consumo_kwh) * 0.95\n    return round(saldo, 2)\n",
        Some(20),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pass_1.verdict, "PASSED");
    assert!(pass_1.reward.is_some());
    assert_eq!(
        pass_1.reward.as_ref().unwrap().next_level_unlocked_id.as_deref(),
        Some("lvl_02_condicionais")
    );
    assert!(pass_1.updated_profile.total_xp > 0);

    // Verifica que fase 1.2 foi desbloqueada atomicamente
    let after_1 = core_get_curriculum(&repo).await.unwrap();
    assert_eq!(after_1.levels[0].status, "COMPLETED_MASTERY");
    assert_eq!(after_1.levels[1].status, "UNLOCKED");

    // Fase 1.2: Condicionais (incluindo bordas 195 e 240)
    let pass_2 = core_submit_code(
        &repo,
        "lvl_02_condicionais",
        "def classificar_tensao(volts):\n    if volts < 195:\n        return 'SUBTENSAO'\n    elif volts > 240:\n        return 'SOBRETENSAO'\n    else:\n        return 'NORMAL'\n",
        Some(25),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pass_2.verdict, "PASSED");

    // Fase 1.3: Laço While
    let pass_3 = core_submit_code(
        &repo,
        "lvl_03_laco_while",
        "def ciclos_ate_meta(carga_inicial, incremento_por_ciclo, meta_maxima):\n    ciclos = 0\n    carga_atual = carga_inicial\n    while carga_atual < meta_maxima:\n        carga_atual += incremento_por_ciclo\n        ciclos += 1\n    return ciclos\n",
        Some(30),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pass_3.verdict, "PASSED");

    // Fase 1.4: Vetores e For
    let pass_4 = core_submit_code(
        &repo,
        "lvl_04_vetores",
        "def filtrar_leituras_validas(leituras):\n    validas = []\n    for x in leituras:\n        if x > 0:\n            validas.append(x)\n    return validas\n",
        Some(30),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pass_4.verdict, "PASSED");

    // Fase 1.5: BOSS Level
    let pass_5 = core_submit_code(
        &repo,
        "lvl_05_boss_modulo1",
        "def auditar_estacao(leituras_kw, limite_alerta):\n    soma = 0\n    alertas = 0\n    for kw in leituras_kw:\n        if kw == 9999:\n            break\n        if kw < 0:\n            continue\n        soma += kw\n        if kw > limite_alerta:\n            alertas += 1\n    return {'total_valido': soma, 'alertas': alertas}\n",
        Some(45),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(pass_5.verdict, "PASSED");
    assert!(pass_5
        .reward
        .as_ref()
        .unwrap()
        .newly_unlocked_badges
        .contains(&"BOSS_SLAYER_M1".to_string()));

    // Exporta carteira e valida assinatura SHA-256
    let export = repo
        .export_signed_wallet("prof_local_01", "mod_01_fundamentos")
        .await
        .unwrap();
    assert_eq!(export.certificate.completed_levels_count, 5);
    assert!(logickiosk_lib::agents::game_master::MasteryGameMasterAgent::verify_wallet_signature(
        &export.certificate
    ));
}

#[tokio::test]
async fn test_gemini_wiremock_200_429_cascade_and_500_graceful_degradation() {
    let mock_server = MockServer::start().await;

    // 1. 200 OK no modelo primário com trecho de código vazado para testar sanitização
    Mock::given(method("POST"))
        .and(path_regex(r".*gemini-3\.8-flash:generateContent.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": json!({
                            "diagnostic_category": "LOGIC_MISMATCH",
                            "friendly_explanation": "Veja: ```python\nreturn a - b\n``` está quase lá.",
                            "socratic_question": "Qual sinal matemático calcula a diferença?",
                            "focus_line": 2,
                            "conceptual_reminder": "Observe a ordem da subtração."
                        }).to_string()
                    }]
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let agent = SocraticTutorAgent::with_base_url(Some("test-key".to_string()), mock_server.uri());
    let rules = LevelRuleConfig {
        level_id: "lvl_01".to_string(),
        level_type: "STANDARD".to_string(),
        problem_statement_md: "Calcule".to_string(),
        must_contain_nodes: vec![],
        forbidden_nodes: vec![],
        base_xp_reward: 100,
        expected_time_seconds: 120,
    };
    let ast_ok = AstInspectionReport {
        is_valid: true,
        detected_nodes: vec![],
        missing_required_nodes: vec![],
        triggered_forbidden_nodes: vec![],
        suspected_hardcode: false,
        syntax_error_detail: None,
    };

    let res = agent
        .generate_socratic_guidance("def f(a,b): return a+b", &rules, &ast_ok, Some("ARITHMETIC_FORMULA"), false)
        .await;
    assert_eq!(res.provider_used, "GEMINI_3_8_FLASH");
    assert!(!res.friendly_explanation.contains("return a - b"));
    assert!(res.friendly_explanation.contains("[trecho de código omitido pelo Mentor]"));

    // 2. Simula 429 no primário -> failover automático para gemini-3.5-flash-lite
    let mock_cascade = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r".*gemini-3\.8-flash:generateContent.*"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_cascade)
        .await;

    Mock::given(method("POST"))
        .and(path_regex(r".*gemini-3\.5-flash-lite:generateContent.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": json!({
                            "diagnostic_category": "LOGIC_MISMATCH",
                            "friendly_explanation": "Fallback acionado com sucesso.",
                            "socratic_question": "O que acontece na linha 2?",
                            "focus_line": 2,
                            "conceptual_reminder": "Verifique os operadores."
                        }).to_string()
                    }]
                }
            }]
        })))
        .mount(&mock_cascade)
        .await;

    let agent_cascade =
        SocraticTutorAgent::with_base_url(Some("test-key".to_string()), mock_cascade.uri());
    let res_cascade = agent_cascade
        .generate_socratic_guidance("def f(): pass", &rules, &ast_ok, None, false)
        .await;
    assert_eq!(res_cascade.provider_used, "GEMINI_3_5_LITE");

    // 3. Simula 500 + JSON malformado -> fallback determinístico Rust Offline
    let mock_fail = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r".*gemini-3\.8-flash:generateContent.*"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_fail)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r".*gemini-3\.5-flash-lite:generateContent.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{ "content": { "parts": [{ "text": "{json quebrado" }] } }]
        })))
        .mount(&mock_fail)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r".*gemma-4-26b-a4b-it:generateContent.*"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_fail)
        .await;

    let agent_fail = SocraticTutorAgent::with_base_url(Some("test-key".to_string()), mock_fail.uri());
    let res_offline = agent_fail
        .generate_socratic_guidance("def f(): pass", &rules, &ast_ok, Some("EXACT_BOUNDARY_INCLUSION_195"), false)
        .await;
    assert_eq!(res_offline.provider_used, "LOCAL_RUST_ENGINE");
    assert!(res_offline.socratic_question.contains("195"));
}

#[test]
fn test_sandbox_edge_cases_timeout_memory_recursion_stdout() {
    let tc = vec![TestCaseSpec {
        id: "tc_edge".to_string(),
        input_data_json: "[5]".to_string(),
        expected_output_json: "5".to_string(),
        is_hidden_edge_case: false,
        failure_concept_tag: "GENERAL".to_string(),
    }];

    // 1. Loop infinito com captura de trace e timeout rápido (150ms no teste)
    let inf_code = "def f(x):\n    i = 0\n    while x > 0:\n        i += 1\n    return i\n";
    let res_timeout = SandboxRunner::execute_test_suite(inf_code, &tc, 150);
    assert_eq!(res_timeout.verdict, "TIMEOUT_LOOP");
    assert!(!res_timeout.variable_trace.is_empty());

    // 2. Bomba de memória ("A" * (10**9)) + truncamento de stdout
    let mem_bomb = "def f(x):\n    s = 'A' * (10**9)\n    return len(s)\n";
    let res_mem = SandboxRunner::execute_test_suite(mem_bomb, &tc, 500);
    assert_eq!(res_mem.verdict, "TEST_FAILED");
    assert!(res_mem.runtime_error.unwrap_or_default().contains("MemoryLimitError"));

    // 3. Truncamento de stdout em 10.000 caracteres
    let big_stdout = "def f(x):\n    for i in range(2500):\n        print('1234567890')\n    return x\n";
    let res_stdout = SandboxRunner::execute_test_suite(big_stdout, &tc, 1000);
    assert!(res_stdout.stdout_truncated);
    assert_eq!(res_stdout.stdout.chars().count(), 10_000);

    // 4. Recursão Infinita -> RecursionError
    let rec_code = "def f(x):\n    return f(x)\n";
    let res_rec = SandboxRunner::execute_test_suite(rec_code, &tc, 500);
    assert_eq!(res_rec.verdict, "TEST_FAILED");
    assert!(res_rec.runtime_error.unwrap_or_default().contains("RecursionError"));

    // 5. Bloqueio de hardcode em AST quando retorna literal de saída
    let rules = LevelRuleConfig {
        level_id: "lvl_02".to_string(),
        level_type: "STANDARD".to_string(),
        problem_statement_md: "".to_string(),
        must_contain_nodes: vec!["FunctionDef".to_string()],
        forbidden_nodes: vec![],
        base_xp_reward: 100,
        expected_time_seconds: 120,
    };
    let tc_par = vec![TestCaseSpec {
        id: "tc_par".to_string(),
        input_data_json: "[2]".to_string(),
        expected_output_json: "\"Par\"".to_string(),
        is_hidden_edge_case: false,
        failure_concept_tag: "PARITY".to_string(),
    }];
    let ast_hardcode = AstJudgeAgent::inspect_code("def f(n):\n    return 'Par'\n", &rules, &tc_par);
    assert!(!ast_hardcode.is_valid);
    assert!(ast_hardcode.suspected_hardcode);
}
