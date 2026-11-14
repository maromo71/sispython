use crate::agents::game_master::MasteryGameMasterAgent;
use crate::agents::judge::AstJudgeAgent;
use crate::agents::socratic::SocraticTutorAgent;
use crate::agents::{
    AstInspectionReport, MasteryRewardPayload, SignedWalletCertificate, SocraticHintResponse,
};
use crate::db::repository::{
    BadgeStatusRecord, DatabaseRepository, LevelProgressRecord, ModuleRecord, ProfileRecord,
    PublicTestCaseView, SignedWalletExport,
};
use crate::sandbox::runner::{
    SandboxExecutionSummary, DEFAULT_MAX_EXECUTION_TIME_MS,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

const KEYRING_SERVICE: &str = "logickiosk";
const KEYRING_ACCOUNT: &str = "gemini_api_key";

pub struct AppState {
    pub repo: Arc<DatabaseRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumPayload {
    pub profile: ProfileRecord,
    pub modules: Vec<ModuleRecord>,
    pub levels: Vec<LevelProgressRecord>,
    pub public_test_cases: HashMap<String, Vec<PublicTestCaseView>>,
    pub badges: Vec<BadgeStatusRecord>,
    pub ai_key_configured: bool,
    pub ai_status_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitCodeResponse {
    pub verdict: String, // "PASSED" | "AST_REJECTED" | "TEST_FAILED" | "TIMEOUT_LOOP" | "SYNTAX_ERROR"
    pub ast_report: AstInspectionReport,
    pub execution_summary: Option<SandboxExecutionSummary>,
    pub socratic_hint: Option<SocraticHintResponse>,
    pub reward: Option<MasteryRewardPayload>,
    pub updated_profile: ProfileRecord,
    pub clock_tamper_detected: bool,
}

pub fn read_api_key_from_keyring() -> Option<String> {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT) {
        if let Ok(secret) = entry.get_password() {
            if !secret.trim().is_empty() {
                return Some(secret);
            }
        }
    }
    std::env::var("GEMINI_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

pub async fn core_get_curriculum(repo: &DatabaseRepository) -> Result<CurriculumPayload, String> {
    let profile = repo.get_default_profile().await?;
    let modules = repo.get_modules().await?;
    let levels = repo.get_levels_with_progress(&profile.id).await?;
    let public_test_cases = repo.get_public_test_cases_for_all_levels().await?;
    let badges = repo.get_badges_status(&profile.id).await?;
    let key_configured = read_api_key_from_keyring().is_some();
    let ai_status_label = if key_configured {
        "Online: Gemini Free".to_string()
    } else {
        "Offline: Motor Nativo Rust".to_string()
    };

    Ok(CurriculumPayload {
        profile,
        modules,
        levels,
        public_test_cases,
        badges,
        ai_key_configured: key_configured,
        ai_status_label,
    })
}

pub async fn core_submit_code(
    repo: &DatabaseRepository,
    level_id: &str,
    source_code: &str,
    elapsed_seconds: Option<i32>,
    api_key_override: Option<String>,
    gemini_base_url_override: Option<String>,
) -> Result<SubmitCodeResponse, String> {
    let profile = repo.get_default_profile().await?;
    let (status, _, _, _, _) = repo
        .get_user_progress_for_level(&profile.id, level_id)
        .await?;

    if status == "LOCKED" {
        return Err(
            "Fase bloqueada! Aprove 100% dos casos públicos, ocultos e regras de AST da fase anterior."
                .to_string(),
        );
    }

    let (rules, module_id, order_index, test_cases) =
        repo.get_level_rules_and_tests(level_id).await?;

    let ast_report = AstJudgeAgent::inspect_code(source_code, &rules, &test_cases);

    let resolved_key = api_key_override.or_else(read_api_key_from_keyring);
    let tutor = match gemini_base_url_override {
        Some(base) => SocraticTutorAgent::with_base_url(resolved_key, base),
        None => SocraticTutorAgent::new(resolved_key),
    };

    if !ast_report.is_valid {
        let verdict = if ast_report.syntax_error_detail.is_some() {
            "SYNTAX_ERROR".to_string()
        } else {
            "AST_REJECTED".to_string()
        };

        let hint = tutor
            .generate_socratic_guidance(source_code, &rules, &ast_report, None, false)
            .await;

        repo.record_failed_submission(
            &profile.id,
            level_id,
            source_code,
            &verdict,
            0,
            "[]",
            &hint.provider_used,
            &hint.diagnostic_category,
            &hint.socratic_question,
            hint.focus_line,
        )
        .await?;

        let updated_profile = repo.get_default_profile().await?;
        return Ok(SubmitCodeResponse {
            verdict,
            ast_report,
            execution_summary: None,
            socratic_hint: Some(hint),
            reward: None,
            updated_profile,
            clock_tamper_detected: false,
        });
    }

    let max_time_ms = std::env::var("SANDBOX_MAX_EXECUTION_TIME_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_EXECUTION_TIME_MS);

    let code_owned = source_code.to_string();
    let tc_owned = test_cases.clone();
    let exec_summary = tokio::task::spawn_blocking(move || {
        AstJudgeAgent::run_test_suite(&code_owned, &tc_owned, max_time_ms)
    })
    .await
    .map_err(|e| format!("Falha interna na thread do Sandbox: {}", e))?;

    let trace_json = serde_json::to_string(&exec_summary.variable_trace).unwrap_or_else(|_| "[]".to_string());

    if !exec_summary.all_passed {
        let is_timeout = exec_summary.verdict == "TIMEOUT_LOOP";
        let hint = tutor
            .generate_socratic_guidance_with_error(
                source_code,
                &rules,
                &ast_report,
                exec_summary.first_failed_concept_tag.as_deref(),
                is_timeout,
                exec_summary.runtime_error.as_deref(),
            )
            .await;

        repo.record_failed_submission(
            &profile.id,
            level_id,
            source_code,
            &exec_summary.verdict,
            exec_summary.execution_time_ms,
            &trace_json,
            &hint.provider_used,
            &hint.diagnostic_category,
            &hint.socratic_question,
            hint.focus_line,
        )
        .await?;

        let updated_profile = repo.get_default_profile().await?;
        return Ok(SubmitCodeResponse {
            verdict: exec_summary.verdict.clone(),
            ast_report,
            execution_summary: Some(exec_summary),
            socratic_hint: Some(hint),
            reward: None,
            updated_profile,
            clock_tamper_detected: false,
        });
    }

    let today_iso = chrono::Local::now().format("%Y-%m-%d").to_string();
    let (reward, clock_tamper) = repo
        .record_passed_submission_atomic(
            &profile.id,
            &module_id,
            level_id,
            order_index,
            &rules.level_type,
            source_code,
            exec_summary.execution_time_ms,
            &trace_json,
            rules.base_xp_reward,
            rules.expected_time_seconds,
            elapsed_seconds.unwrap_or(60),
            &today_iso,
        )
        .await?;

    let updated_profile = repo.get_default_profile().await?;

    Ok(SubmitCodeResponse {
        verdict: "PASSED".to_string(),
        ast_report,
        execution_summary: Some(exec_summary),
        socratic_hint: None,
        reward: Some(reward),
        updated_profile,
        clock_tamper_detected: clock_tamper,
    })
}

#[tauri::command]
pub async fn get_curriculum(state: State<'_, AppState>) -> Result<CurriculumPayload, String> {
    core_get_curriculum(&state.repo).await
}

#[tauri::command]
pub async fn submit_code(
    state: State<'_, AppState>,
    level_id: String,
    source_code: String,
    elapsed_seconds: Option<i32>,
) -> Result<SubmitCodeResponse, String> {
    core_submit_code(&state.repo, &level_id, &source_code, elapsed_seconds, None, None).await
}

#[tauri::command]
pub async fn request_socratic_hint(
    state: State<'_, AppState>,
    level_id: String,
    source_code: String,
    failed_concept_tag: Option<String>,
) -> Result<SocraticHintResponse, String> {
    let profile = state.repo.get_default_profile().await?;
    let (rules, _, _, test_cases) = state.repo.get_level_rules_and_tests(&level_id).await?;

    if rules.level_type == "BOSS" {
        return Ok(SocraticHintResponse {
            diagnostic_category: "BOSS_RESTRICTION".to_string(),
            friendly_explanation: "Pistas socráticas proativas são bloqueadas em Desafios Chefão.".to_string(),
            socratic_question: "Combine os conceitos de laços, condicionais e parada por sentinela que você dominou nas fases anteriores.".to_string(),
            focus_line: 1,
            conceptual_reminder: "Fases Chefão exigem autonomia completa.".to_string(),
            provider_used: "LOCAL_RUST_ENGINE".to_string(),
        });
    }

    state.repo.increment_hint_usage(&profile.id, &level_id).await?;

    let ast_report = AstJudgeAgent::inspect_code(&source_code, &rules, &test_cases);
    let tutor = SocraticTutorAgent::new(read_api_key_from_keyring());
    let hint = tutor
        .generate_socratic_guidance(
            &source_code,
            &rules,
            &ast_report,
            failed_concept_tag.as_deref(),
            false,
        )
        .await;

    Ok(hint)
}

#[tauri::command]
pub async fn toggle_theme(state: State<'_, AppState>, theme: String) -> Result<String, String> {
    let profile = state.repo.get_default_profile().await?;
    state.repo.update_theme(&profile.id, &theme).await?;
    Ok(theme)
}

#[tauri::command]
pub async fn update_student_name(
    state: State<'_, AppState>,
    display_name: String,
) -> Result<ProfileRecord, String> {
    let profile = state.repo.get_default_profile().await?;
    state
        .repo
        .update_display_name(&profile.id, &display_name)
        .await?;
    state.repo.get_default_profile().await
}

#[tauri::command]
pub async fn save_api_key(api_key: String) -> Result<bool, String> {
    let trimmed = api_key.trim();
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| format!("Falha ao acessar cofre do SO: {}", e))?;
    if trimmed.is_empty() {
        let _ = entry.delete_credential();
        return Ok(false);
    }
    entry
        .set_password(trimmed)
        .map_err(|e| format!("Falha ao salvar chave no Keyring do SO: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub async fn get_api_key_status() -> Result<bool, String> {
    Ok(read_api_key_from_keyring().is_some())
}

#[tauri::command]
pub async fn export_signed_wallet(
    state: State<'_, AppState>,
    module_id: String,
) -> Result<SignedWalletExport, String> {
    let profile = state.repo.get_default_profile().await?;
    state.repo.export_signed_wallet(&profile.id, &module_id).await
}

#[tauri::command]
pub async fn verify_signed_wallet(certificate: SignedWalletCertificate) -> Result<bool, String> {
    Ok(MasteryGameMasterAgent::verify_wallet_signature(&certificate))
}
