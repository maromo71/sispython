use crate::agents::{LevelRuleConfig, TestCaseSpec};
use crate::sandbox::ast_inspector::{AstInspector, SUPPORTED_AST_NODES};
use crate::sandbox::runner::{SandboxRunner, DEFAULT_MAX_EXECUTION_TIME_MS};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

pub const MIN_EXERCISES_PER_MODULE: usize = 5;
pub const MAX_EXERCISES_PER_MODULE: usize = 20;
const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const BATCH_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestCaseDraft {
    pub input_data_json: String,
    pub expected_output_json: String,
    pub is_hidden_edge_case: bool,
    pub failure_concept_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiGeneratedLevelDraft {
    pub title: String,
    pub level_type: String,
    pub problem_statement_md: String,
    pub starter_code: String,
    pub must_contain_nodes: Vec<String>,
    pub forbidden_nodes: Vec<String>,
    pub base_xp_reward: i32,
    pub expected_time_seconds: i32,
    pub reference_solution: String,
    pub test_cases: Vec<GeneratedTestCaseDraft>,
}

/// Especificação validada no AST + Sandbox (sem `reference_solution`, nunca persistida).
#[derive(Debug, Clone)]
pub struct VerifiedLevelSpec {
    pub order_index: i32,
    pub title: String,
    pub level_type: String,
    pub problem_statement_md: String,
    pub starter_code: String,
    pub must_contain_nodes: Vec<String>,
    pub forbidden_nodes: Vec<String>,
    pub base_xp_reward: i32,
    pub expected_time_seconds: i32,
    pub test_cases: Vec<GeneratedTestCaseDraft>,
}

#[derive(Debug, Clone)]
pub struct CurriculumBatchRequest {
    pub module_title: String,
    pub module_description: String,
    pub topic: String,
    pub difficulty: String,
    pub total_exercises: usize,
    pub start_order: usize,
    pub end_order: usize,
    pub include_boss_at_end: bool,
    pub existing_titles: Vec<String>,
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeneratedLevelsResult {
    pub levels: Vec<VerifiedLevelSpec>,
    pub provider_used: String,
}

pub struct CurriculumGeneratorAgent {
    http_client: Client,
    api_key: Option<String>,
    base_url: String,
    ollama_endpoint: Option<String>,
    ollama_model: String,
}

impl CurriculumGeneratorAgent {
    pub fn new(api_key: Option<String>) -> Self {
        let base_url = std::env::var("GEMINI_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_GEMINI_BASE_URL.to_string());
        Self::with_base_url(api_key, base_url)
    }

    pub fn with_base_url(api_key: Option<String>, base_url: String) -> Self {
        let timeout_ms = std::env::var("GEMINI_CURRICULUM_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(45_000);

        let http_client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_else(|_| Client::new());

        let ollama_enabled = std::env::var("LOCAL_LLM_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let ollama_endpoint = if ollama_enabled {
            Some(
                std::env::var("LOCAL_LLM_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string()),
            )
        } else {
            None
        };
        let ollama_model =
            std::env::var("LOCAL_LLM_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());

        Self {
            http_client,
            api_key,
            base_url,
            ollama_endpoint,
            ollama_model,
        }
    }

    pub fn validate_exercise_count(count: usize) -> Result<(), String> {
        if !(MIN_EXERCISES_PER_MODULE..=MAX_EXERCISES_PER_MODULE).contains(&count) {
            return Err(format!(
                "Cada módulo deve possuir entre {} e {} exercícios (recebido: {}).",
                MIN_EXERCISES_PER_MODULE, MAX_EXERCISES_PER_MODULE, count
            ));
        }
        Ok(())
    }

    pub async fn generate_module_levels(
        &self,
        module_title: &str,
        module_description: &str,
        topic: &str,
        difficulty: &str,
        total_exercises: usize,
        start_order: usize,
        include_boss_at_end: bool,
        mut existing_titles: Vec<String>,
        custom_instructions: Option<String>,
    ) -> Result<GeneratedLevelsResult, String> {
        Self::validate_exercise_count(total_exercises)?;
        if start_order == 0 || start_order > total_exercises {
            return Err("Faixa de ordem inválida para geração de exercícios.".to_string());
        }

        let mut all_verified = Vec::new();
        let mut last_provider = String::new();
        let mut cur = start_order;

        while cur <= total_exercises {
            let batch_end = (cur + BATCH_SIZE - 1).min(total_exercises);
            let batch_has_boss = include_boss_at_end && batch_end == total_exercises;

            let req = CurriculumBatchRequest {
                module_title: module_title.to_string(),
                module_description: module_description.to_string(),
                topic: topic.to_string(),
                difficulty: difficulty.to_string(),
                total_exercises,
                start_order: cur,
                end_order: batch_end,
                include_boss_at_end: batch_has_boss,
                existing_titles: existing_titles.clone(),
                custom_instructions: custom_instructions.clone(),
            };

            let (verified_batch, provider) = self.generate_and_verify_batch(&req).await?;
            for lvl in &verified_batch {
                existing_titles.push(lvl.title.clone());
            }
            all_verified.extend(verified_batch);
            last_provider = provider;
            cur = batch_end + 1;
        }

        Ok(GeneratedLevelsResult {
            levels: all_verified,
            provider_used: last_provider,
        })
    }

    async fn generate_and_verify_batch(
        &self,
        req: &CurriculumBatchRequest,
    ) -> Result<(Vec<VerifiedLevelSpec>, String), String> {
        let mut correction_feedback: Option<String> = None;
        let mut last_error = String::new();

        for _attempt in 0..2 {
            let (drafts, provider) = self
                .request_ai_drafts(req, correction_feedback.as_deref())
                .await?;

            match Self::verify_batch_drafts(req, drafts) {
                Ok(verified) => return Ok((verified, provider)),
                Err(err) => {
                    last_error = err.clone();
                    correction_feedback = Some(err);
                }
            }
        }

        Err(format!(
            "A IA gerou exercícios incompatíveis com o Sandbox após tentativa de autocorreção: {}",
            last_error
        ))
    }

    fn verify_batch_drafts(
        req: &CurriculumBatchRequest,
        drafts: Vec<AiGeneratedLevelDraft>,
    ) -> Result<Vec<VerifiedLevelSpec>, String> {
        let expected_len = req.end_order - req.start_order + 1;
        if drafts.len() != expected_len {
            return Err(format!(
                "Esperava {} exercícios (ordens {}..={}), mas a IA retornou {}.",
                expected_len, req.start_order, req.end_order, drafts.len()
            ));
        }

        let mut verified = Vec::with_capacity(expected_len);
        for (idx, draft) in drafts.into_iter().enumerate() {
            let order = req.start_order + idx;
            let is_boss = req.include_boss_at_end && order == req.total_exercises;
            let spec = Self::verify_single_draft(order, is_boss, draft)?;
            verified.push(spec);
        }

        Ok(verified)
    }

    fn verify_single_draft(
        order: usize,
        is_boss: bool,
        draft: AiGeneratedLevelDraft,
    ) -> Result<VerifiedLevelSpec, String> {
        let level_type = if is_boss {
            "BOSS".to_string()
        } else if draft.level_type.eq_ignore_ascii_case("FIXATION") {
            "FIXATION".to_string()
        } else {
            "STANDARD".to_string()
        };

        let mut must_nodes: Vec<String> = draft
            .must_contain_nodes
            .into_iter()
            .filter(|n| SUPPORTED_AST_NODES.contains(&n.as_str()))
            .collect();
        if !must_nodes.contains(&"FunctionDef".to_string()) {
            must_nodes.insert(0, "FunctionDef".to_string());
        }
        if !must_nodes.contains(&"Return".to_string()) {
            must_nodes.push("Return".to_string());
        }
        // Evita exigir nós proibidos ou inválidos por acidente
        must_nodes.retain(|n| !matches!(n.as_str(), "Import" | "Eval" | "Exec"));
        must_nodes.sort();
        must_nodes.dedup();

        let mut forbidden_nodes: Vec<String> = draft
            .forbidden_nodes
            .into_iter()
            .filter(|n| SUPPORTED_AST_NODES.contains(&n.as_str()) && !must_nodes.contains(n))
            .collect();
        for required_forbidden in ["Import", "Eval", "Exec"] {
            if !forbidden_nodes.iter().any(|n| n == required_forbidden) {
                forbidden_nodes.push(required_forbidden.to_string());
            }
        }

        if draft.test_cases.len() < 2 {
            return Err(format!(
                "Fase {} ('{}') precisa de pelo menos 2 casos de teste.",
                order, draft.title
            ));
        }

        let mut clean_tests = Vec::with_capacity(draft.test_cases.len());
        for (tc_idx, mut tc) in draft.test_cases.into_iter().enumerate() {
            let parsed_input: Value = serde_json::from_str(&tc.input_data_json).map_err(|e| {
                format!(
                    "Fase {} ('{}'): input_data_json inválido no teste {}: {}",
                    order,
                    draft.title,
                    tc_idx + 1,
                    e
                )
            })?;
            if !parsed_input.is_array() {
                return Err(format!(
                    "Fase {} ('{}'): input_data_json do teste {} deve ser um array JSON de argumentos.",
                    order,
                    draft.title,
                    tc_idx + 1
                ));
            }
            let _: Value = serde_json::from_str(&tc.expected_output_json).map_err(|e| {
                format!(
                    "Fase {} ('{}'): expected_output_json inválido no teste {}: {}",
                    order,
                    draft.title,
                    tc_idx + 1,
                    e
                )
            })?;
            if tc.failure_concept_tag.trim().is_empty() {
                tc.failure_concept_tag = "GENERAL_LOGIC".to_string();
            }
            clean_tests.push(tc);
        }

        // Garante ao menos 1 teste público e 1 oculto
        if !clean_tests.iter().any(|t| !t.is_hidden_edge_case) {
            clean_tests[0].is_hidden_edge_case = false;
        }
        if !clean_tests.iter().any(|t| t.is_hidden_edge_case) {
            if let Some(last) = clean_tests.last_mut() {
                last.is_hidden_edge_case = true;
            }
        }

        let test_specs: Vec<TestCaseSpec> = clean_tests
            .iter()
            .enumerate()
            .map(|(i, tc)| TestCaseSpec {
                id: format!("verify_{}_{}", order, i),
                input_data_json: tc.input_data_json.clone(),
                expected_output_json: tc.expected_output_json.clone(),
                is_hidden_edge_case: tc.is_hidden_edge_case,
                failure_concept_tag: tc.failure_concept_tag.clone(),
            })
            .collect();

        let rules = LevelRuleConfig {
            level_id: format!("verify_{}", order),
            level_type: level_type.clone(),
            problem_statement_md: draft.problem_statement_md.clone(),
            must_contain_nodes: must_nodes.clone(),
            forbidden_nodes: forbidden_nodes.clone(),
            base_xp_reward: draft.base_xp_reward.clamp(100, 1000),
            expected_time_seconds: draft.expected_time_seconds.clamp(120, 600),
        };

        let ast_report = AstInspector::inspect(&draft.reference_solution, &rules, &test_specs);
        if !ast_report.is_valid {
            return Err(format!(
                "Fase {} ('{}') reprovou na validação AST da solução de referência: syntax={:?}, missing={:?}, forbidden={:?}, hardcode={}",
                order,
                draft.title,
                ast_report.syntax_error_detail,
                ast_report.missing_required_nodes,
                ast_report.triggered_forbidden_nodes,
                ast_report.suspected_hardcode
            ));
        }

        let exec_summary = SandboxRunner::execute_test_suite(
            &draft.reference_solution,
            &test_specs,
            DEFAULT_MAX_EXECUTION_TIME_MS,
        );
        if !exec_summary.all_passed {
            let failed_detail = exec_summary
                .runtime_error
                .or_else(|| {
                    exec_summary
                        .test_results
                        .iter()
                        .find(|t| !t.passed)
                        .and_then(|t| t.error_message.clone().or_else(|| {
                            Some(format!(
                                "esperado {:?}, obtido {:?}",
                                t.expected_preview, t.actual_preview
                            ))
                        }))
                })
                .unwrap_or_else(|| exec_summary.verdict.clone());
            return Err(format!(
                "Fase {} ('{}') falhou na execução de verificação do Sandbox: {}",
                order, draft.title, failed_detail
            ));
        }

        // reference_solution é descartada aqui e nunca persiste no banco ou IPC
        let default_xp = if is_boss {
            500 + (order as i32 * 30)
        } else {
            100 + (order as i32 * 35)
        };

        Ok(VerifiedLevelSpec {
            order_index: order as i32,
            title: draft.title.trim().to_string(),
            level_type,
            problem_statement_md: draft.problem_statement_md.trim().to_string(),
            starter_code: if draft.starter_code.trim().is_empty() {
                "def resolver(entrada):\n    return None\n".to_string()
            } else {
                draft.starter_code
            },
            must_contain_nodes: must_nodes,
            forbidden_nodes,
            base_xp_reward: if draft.base_xp_reward > 0 {
                draft.base_xp_reward.clamp(100, 1200)
            } else {
                default_xp
            },
            expected_time_seconds: if draft.expected_time_seconds > 0 {
                draft.expected_time_seconds.clamp(120, 600)
            } else if is_boss {
                360
            } else {
                210
            },
            test_cases: clean_tests,
        })
    }

    async fn request_ai_drafts(
        &self,
        req: &CurriculumBatchRequest,
        correction_feedback: Option<&str>,
    ) -> Result<(Vec<AiGeneratedLevelDraft>, String), String> {
        let mut errors = Vec::new();

        if let Some(key) = &self.api_key {
            if !key.trim().is_empty() {
                let primary = std::env::var("GEMINI_PRIMARY_FREE_MODEL")
                    .unwrap_or_else(|_| "gemini-3.8-flash".to_string());
                let fallback = std::env::var("GEMINI_FALLBACK_FREE_MODEL")
                    .unwrap_or_else(|_| "gemini-3.5-flash-lite".to_string());
                let open_model = std::env::var("GEMINI_OPEN_FREE_MODEL")
                    .unwrap_or_else(|_| "gemma-4-26b-a4b-it".to_string());

                let free_models = [
                    (primary.as_str(), "GEMINI_3_8_FLASH"),
                    (fallback.as_str(), "GEMINI_3_5_LITE"),
                    (open_model.as_str(), "GEMMA_4_26B"),
                ];

                for (model_name, provider_tag) in free_models {
                    match self
                        .call_gemini(key, model_name, req, correction_feedback)
                        .await
                    {
                        Ok(drafts) => return Ok((drafts, provider_tag.to_string())),
                        Err(e) => errors.push(format!("{}: {}", model_name, e)),
                    }
                }
            }
        }

        if let Some(endpoint) = &self.ollama_endpoint {
            match self.call_ollama(endpoint, req, correction_feedback).await {
                Ok(drafts) => return Ok((drafts, "LOCAL_OLLAMA".to_string())),
                Err(e) => errors.push(format!("Ollama: {}", e)),
            }
        }

        if errors.is_empty() {
            return Err(
                "Nenhuma IA configurada. Clique em 'Chave IA' no topo para configurar sua chave gratuita do Google AI Studio ou habilite LOCAL_LLM_ENABLED (Ollama)."
                    .to_string(),
            );
        }

        Err(format!(
            "Não foi possível gerar exercícios com os provedores disponíveis ({}).",
            errors.join(" | ")
        ))
    }

    fn build_prompts(
        req: &CurriculumBatchRequest,
        correction_feedback: Option<&str>,
    ) -> (String, String) {
        let count = req.end_order - req.start_order + 1;
        let allowed_ast = SUPPORTED_AST_NODES
            .iter()
            .filter(|n| !matches!(**n, "Import" | "Eval" | "Exec"))
            .copied()
            .collect::<Vec<_>>()
            .join(", ");

        let system_prompt = format!(
            "Você é um Arquiteto Pedagógico de Python do LogicKiosk. Responda APENAS em JSON válido seguindo o schema.\n\
            REGRAS TÉCNICAS INVIOLÁVEIS DO SANDBOX:\n\
            1. O Sandbox NÃO permite imports (import math, re, collections etc. são PROIBIDOS) nem lambda/set/try complexos ou bibliotecas externas.\n\
            2. Recursos Python suportados pelo Sandbox: def, return, if/elif/else, while, for, break, continue, atribuições, list, tuple, dict, list comprehension, slicing, len, range, round, abs, min, max, sum, int, float, str, bool, enumerate, sorted (somente números), reversed, any, all; métodos de str (.upper, .lower, .strip, .split, .join), dict (.get, .keys, .values, .items), list (.append, .pop, .extend, .clear, .reverse, .sort, .count, .index).\n\
            3. `must_contain_nodes` e `forbidden_nodes` DEVEM usar EXCLUSIVAMENTE tags desta lista exata: [{allowed_ast}]. Sempre inclua \"FunctionDef\" e \"Return\" em `must_contain_nodes` e [\"Import\", \"Eval\", \"Exec\"] em `forbidden_nodes`.\n\
            4. Todo exercício DEVE ter `starter_code` com uma função `def nome(params):` compatível com `reference_solution`.\n\
            5. Cada exercício DEVE ter de 2 a 4 `test_cases`, com pelo menos 1 público (`is_hidden_edge_case: false`) e 1 oculto de borda (`is_hidden_edge_case: true`).\n\
            6. `input_data_json` é OBRIGATORIAMENTE uma string contendo um JSON array com os argumentos posicionais da função (ex.: \"[[10, 20], 5]\"). `expected_output_json` é uma string JSON válida (ex.: \"30\", \"\\\"OK\\\"\", \"{{\\\"total\\\": 2}}\").\n\
            7. `reference_solution` deve ser Python 100% executável, passar em TODOS os `test_cases` e conter todos os nós de `must_contain_nodes` (não retornar literal fixo sem `BinOp`/`Compare`/`For`/`While`).\n\
            8. Não repita exercícios já existentes no módulo."
        );

        let boss_instruction = if req.include_boss_at_end {
            format!(
                "A última fase deste lote (ordem {}) OBRIGATORIAMENTE deve ter `level_type: \"BOSS\"`, título iniciando com 'DESAFIO CHEFÃO:', combinando os conceitos do módulo sem pistas socráticas.",
                req.end_order
            )
        } else {
            "Todas as fases deste lote devem ter `level_type: \"STANDARD\"` ou `\"FIXATION\"` (nenhuma BOSS neste lote).".to_string()
        };

        let existing_list = if req.existing_titles.is_empty() {
            "Nenhum ainda".to_string()
        } else {
            req.existing_titles.join("; ")
        };

        let custom_note = req
            .custom_instructions
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Seguir progressão prática e realista em Português do Brasil.");

        let retry_note = correction_feedback
            .map(|fb| format!("\nCORREÇÃO OBRIGATÓRIA DA TENTATIVA ANTERIOR QUE FALHOU:\n{}\n", fb))
            .unwrap_or_default();

        let user_prompt = format!(
            "Gere exatamente {count} exercícios (ordens {start} até {end} de um módulo com {total} fases) em Português do Brasil.\n\
            Módulo: {title}\n\
            Descrição: {desc}\n\
            Tema Central: {topic}\n\
            Nível de Dificuldade: {diff}\n\
            Instruções Adicionais: {custom_note}\n\
            Títulos já existentes (não duplicar): {existing_list}\n\
            Regra de Boss: {boss_instruction}{retry_note}\n\
            Retorne um objeto JSON `{{\"levels\": [...]}}` com exatamente {count} itens na ordem crescente.",
            count = count,
            start = req.start_order,
            end = req.end_order,
            total = req.total_exercises,
            title = req.module_title,
            desc = req.module_description,
            topic = req.topic,
            diff = req.difficulty,
        );

        (system_prompt, user_prompt)
    }

    async fn call_gemini(
        &self,
        api_key: &str,
        model: &str,
        req: &CurriculumBatchRequest,
        correction_feedback: Option<&str>,
    ) -> Result<Vec<AiGeneratedLevelDraft>, String> {
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/{}:generateContent?key={}", base, model, api_key);
        let (system_prompt, user_prompt) = Self::build_prompts(req, correction_feedback);

        let payload = json!({
            "systemInstruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": user_prompt }]
            }],
            "generationConfig": {
                "temperature": 0.35,
                "responseMimeType": "application/json",
                "responseSchema": {
                    "type": "OBJECT",
                    "properties": {
                        "levels": {
                            "type": "ARRAY",
                            "items": {
                                "type": "OBJECT",
                                "properties": {
                                    "title": { "type": "STRING" },
                                    "level_type": { "type": "STRING" },
                                    "problem_statement_md": { "type": "STRING" },
                                    "starter_code": { "type": "STRING" },
                                    "must_contain_nodes": {
                                        "type": "ARRAY",
                                        "items": { "type": "STRING" }
                                    },
                                    "forbidden_nodes": {
                                        "type": "ARRAY",
                                        "items": { "type": "STRING" }
                                    },
                                    "base_xp_reward": { "type": "INTEGER" },
                                    "expected_time_seconds": { "type": "INTEGER" },
                                    "reference_solution": { "type": "STRING" },
                                    "test_cases": {
                                        "type": "ARRAY",
                                        "items": {
                                            "type": "OBJECT",
                                            "properties": {
                                                "input_data_json": { "type": "STRING" },
                                                "expected_output_json": { "type": "STRING" },
                                                "is_hidden_edge_case": { "type": "BOOLEAN" },
                                                "failure_concept_tag": { "type": "STRING" }
                                            },
                                            "required": [
                                                "input_data_json",
                                                "expected_output_json",
                                                "is_hidden_edge_case",
                                                "failure_concept_tag"
                                            ]
                                        }
                                    }
                                },
                                "required": [
                                    "title",
                                    "level_type",
                                    "problem_statement_md",
                                    "starter_code",
                                    "must_contain_nodes",
                                    "forbidden_nodes",
                                    "base_xp_reward",
                                    "expected_time_seconds",
                                    "reference_solution",
                                    "test_cases"
                                ]
                            }
                        }
                    },
                    "required": ["levels"]
                }
            }
        });

        let res = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("HTTP {}", res.status()));
        }

        let body: Value = res.json().await.map_err(|e| e.to_string())?;
        let text_json = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Payload Gemini sem campo text")?;

        Self::parse_levels_json(text_json)
    }

    async fn call_ollama(
        &self,
        endpoint: &str,
        req: &CurriculumBatchRequest,
        correction_feedback: Option<&str>,
    ) -> Result<Vec<AiGeneratedLevelDraft>, String> {
        let (system_prompt, user_prompt) = Self::build_prompts(req, correction_feedback);
        let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        let payload = json!({
            "model": self.ollama_model,
            "prompt": full_prompt,
            "format": "json",
            "stream": false
        });

        let res = self
            .http_client
            .post(endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Ollama HTTP {}", res.status()));
        }

        let body: Value = res.json().await.map_err(|e| e.to_string())?;
        let text_json = body["response"].as_str().ok_or("Resposta Ollama vazia")?;

        Self::parse_levels_json(text_json)
    }

    fn parse_levels_json(raw: &str) -> Result<Vec<AiGeneratedLevelDraft>, String> {
        let trimmed = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        #[derive(Deserialize)]
        struct Envelope {
            levels: Vec<AiGeneratedLevelDraft>,
        }

        let parsed: Envelope = serde_json::from_str(trimmed)
            .map_err(|e| format!("JSON de exercícios inválido: {}", e))?;
        Ok(parsed.levels)
    }
}
