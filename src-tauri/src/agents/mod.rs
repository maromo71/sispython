pub mod game_master;
pub mod judge;
pub mod offline_rules;
pub mod socratic;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelRuleConfig {
    pub level_id: String,
    pub level_type: String, // "STANDARD" | "FIXATION" | "BOSS"
    pub problem_statement_md: String,
    pub must_contain_nodes: Vec<String>,
    pub forbidden_nodes: Vec<String>,
    pub base_xp_reward: i32,
    pub expected_time_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseSpec {
    pub id: String,
    pub input_data_json: String,
    pub expected_output_json: String,
    pub is_hidden_edge_case: bool,
    pub failure_concept_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstInspectionReport {
    pub is_valid: bool,
    pub detected_nodes: Vec<String>,
    pub missing_required_nodes: Vec<String>,
    pub triggered_forbidden_nodes: Vec<String>,
    pub suspected_hardcode: bool,
    pub syntax_error_detail: Option<(String, usize)>, // (Mensagem, Linha)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticHintResponse {
    pub diagnostic_category: String,
    pub friendly_explanation: String,
    pub socratic_question: String,
    pub focus_line: usize,
    pub conceptual_reminder: String,
    pub provider_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryRewardPayload {
    pub earned_xp: i32,
    pub new_total_xp: i32,
    pub updated_streak_days: i32,
    pub next_level_unlocked_id: Option<String>,
    pub scheduled_fixation_review: bool,
    pub newly_unlocked_badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedWalletCertificate {
    pub student_name: String,
    pub profile_id: String,
    pub module_id: String,
    pub completed_levels_count: usize,
    pub total_earned_xp: i32,
    pub issued_at_iso: String,
    pub sha256_signature: String,
}
