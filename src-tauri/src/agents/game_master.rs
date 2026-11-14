use crate::agents::{MasteryRewardPayload, SignedWalletCertificate};
use sha2::{Digest, Sha256};

const INTEGRITY_APPLICATION_SALT: &str = "LOGICKIOSK_ACADEMIC_VERIFICATION_V1_SALT";

pub struct MasteryGameMasterAgent;

impl MasteryGameMasterAgent {
    /// Calcula o XP ganho com base na fórmula determinística da especificação:
    /// XP_ganho = floor(XP_base * M_tentativa * M_pistas * M_streak)
    pub fn calculate_xp_reward(
        base_xp: i32,
        attempts_count: i32,
        hints_used_count: i32,
        current_streak_days: i32,
    ) -> i32 {
        let m_attempt = if attempts_count <= 1 {
            1.25
        } else if attempts_count <= 3 {
            1.0
        } else {
            0.80
        };

        let m_hints = if hints_used_count == 0 {
            1.0
        } else if hints_used_count == 1 {
            0.85
        } else {
            0.70
        };

        let m_streak = (1.0 + (current_streak_days.max(0) as f64 * 0.05)).min(1.50);

        let raw_xp = (base_xp as f64) * m_attempt * m_hints * m_streak;
        raw_xp.floor() as i32
    }

    /// Avalia se o aluno precisa de um Desafio Relâmpago de Fixação antes do Boss Level
    pub fn should_schedule_fixation_review(
        attempts_count: i32,
        resolution_time_seconds: i32,
        expected_time_seconds: i32,
    ) -> bool {
        attempts_count >= 5 || resolution_time_seconds > (expected_time_seconds * 3)
    }

    /// Atualiza o streak respeitando proteção anti-cheat de manipulação do relógio do sistema
    /// Retorna (novo_streak, nova_last_activity_date, clock_tamper_detected)
    pub fn compute_streak_update(
        current_streak_days: i32,
        last_activity_date: Option<&str>,
        today_iso_date: &str,
    ) -> (i32, String, bool) {
        let Some(last) = last_activity_date else {
            return (current_streak_days.max(1), today_iso_date.to_string(), false);
        };

        let last_date = chrono::NaiveDate::parse_from_str(last.trim(), "%Y-%m-%d");
        let today_date = chrono::NaiveDate::parse_from_str(today_iso_date.trim(), "%Y-%m-%d");

        match (last_date, today_date) {
            (Ok(prev), Ok(curr)) => {
                if curr < prev {
                    // Relógio retrocedido: congela streak e sinaliza alerta sem corromper contagem
                    (current_streak_days.max(1), last.to_string(), true)
                } else if curr == prev {
                    (current_streak_days.max(1), last.to_string(), false)
                } else if curr == prev + chrono::Days::new(1) {
                    (current_streak_days.max(0) + 1, today_iso_date.to_string(), false)
                } else {
                    (1, today_iso_date.to_string(), false)
                }
            }
            _ => (current_streak_days.max(1), today_iso_date.to_string(), false),
        }
    }

    /// Consolida o payload de recompensa ao aprovar 100% dos testes e AST da fase
    #[allow(clippy::too_many_arguments)]
    pub fn build_mastery_reward(
        base_xp: i32,
        current_total_xp: i32,
        attempts_count: i32,
        hints_used_count: i32,
        current_streak_days: i32,
        resolution_time_seconds: i32,
        expected_time_seconds: i32,
        is_boss_level: bool,
        next_level_id: Option<String>,
    ) -> MasteryRewardPayload {
        let earned_xp = Self::calculate_xp_reward(
            base_xp,
            attempts_count,
            hints_used_count,
            current_streak_days,
        );

        let scheduled_fixation = Self::should_schedule_fixation_review(
            attempts_count,
            resolution_time_seconds,
            expected_time_seconds,
        );

        let mut badges = Vec::new();
        if attempts_count == 1 {
            badges.push("FIRST_CIRCUIT".to_string());
        }
        if is_boss_level && hints_used_count == 0 {
            badges.push("BOSS_SLAYER_M1".to_string());
        }

        MasteryRewardPayload {
            earned_xp,
            new_total_xp: current_total_xp + earned_xp,
            updated_streak_days: current_streak_days.max(1),
            next_level_unlocked_id: next_level_id,
            scheduled_fixation_review: scheduled_fixation,
            newly_unlocked_badges: badges,
        }
    }

    /// Calcula o rank do perfil a partir do XP total
    pub fn compute_level_rank(total_xp: i32) -> i32 {
        (total_xp.max(0) / 250) + 1
    }

    /// Gera uma Carteira de Progresso Assinada Criptograficamente via SHA-256
    pub fn generate_signed_wallet(
        student_name: &str,
        profile_id: &str,
        module_id: &str,
        completed_levels_count: usize,
        total_earned_xp: i32,
        issued_at_iso: &str,
    ) -> SignedWalletCertificate {
        let canonical_payload = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            student_name.trim(),
            profile_id,
            module_id,
            completed_levels_count,
            total_earned_xp,
            issued_at_iso,
            INTEGRITY_APPLICATION_SALT
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical_payload.as_bytes());
        let signature = hex::encode(hasher.finalize());

        SignedWalletCertificate {
            student_name: student_name.trim().to_string(),
            profile_id: profile_id.to_string(),
            module_id: module_id.to_string(),
            completed_levels_count,
            total_earned_xp,
            issued_at_iso: issued_at_iso.to_string(),
            sha256_signature: signature,
        }
    }

    /// Verifica se uma Carteira de Progresso apresentada a um professor é autêntica e íntegra
    pub fn verify_wallet_signature(cert: &SignedWalletCertificate) -> bool {
        let expected = Self::generate_signed_wallet(
            &cert.student_name,
            &cert.profile_id,
            &cert.module_id,
            cert.completed_levels_count,
            cert.total_earned_xp,
            &cert.issued_at_iso,
        );
        expected.sha256_signature == cert.sha256_signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::judge::AstJudgeAgent;
    use crate::agents::socratic::SocraticTutorAgent;
    use crate::agents::{LevelRuleConfig, TestCaseSpec};

    #[test]
    fn test_xp_calculation_formula_first_try_no_hints() {
        // Base 100 * 1.25 (1ª tentativa) * 1.0 (0 dicas) * 1.10 (2 dias de streak) = 137.5 -> 137
        let xp = MasteryGameMasterAgent::calculate_xp_reward(100, 1, 0, 2);
        assert_eq!(xp, 137);
    }

    #[test]
    fn test_xp_calculation_formula_bounds_and_caps() {
        // Streak cap: 20 days -> 1.0 + 1.0 = 2.0 capped at 1.50
        // Attempt 5 -> 0.80, Hints 3 -> 0.70
        // 200 * 0.80 * 0.70 * 1.50 = 168.0
        let xp = MasteryGameMasterAgent::calculate_xp_reward(200, 5, 3, 20);
        assert_eq!(xp, 168);
    }

    #[test]
    fn test_ast_judge_rejects_missing_while_loop() {
        let rules = LevelRuleConfig {
            level_id: "lvl_03_laco_while".to_string(),
            level_type: "STANDARD".to_string(),
            problem_statement_md: "".to_string(),
            must_contain_nodes: vec!["FunctionDef".to_string(), "While".to_string()],
            forbidden_nodes: vec!["For".to_string()],
            base_xp_reward: 200,
            expected_time_seconds: 240,
        };
        let test_cases = vec![TestCaseSpec {
            id: "tc1".to_string(),
            input_data_json: "[20, 15, 80]".to_string(),
            expected_output_json: "4".to_string(),
            is_hidden_edge_case: false,
            failure_concept_tag: "WHILE".to_string(),
        }];

        let hardcoded_code = "def ciclos_ate_meta(a, b, c):\n    return 4\n";
        let report = AstJudgeAgent::inspect_code(hardcoded_code, &rules, &test_cases);

        assert!(!report.is_valid);
        assert!(report.missing_required_nodes.contains(&"While".to_string()));
        assert!(report.suspected_hardcode);
    }

    #[test]
    fn test_socratic_sanitizer_removes_code_blocks() {
        let raw_llm_output = "Olhe para sua linha 3: ```python\nwhile x < 10:\n    x += 1\n``` Tente ajustar isso.";
        let clean = SocraticTutorAgent::sanitize_no_code_leak(raw_llm_output);
        assert!(!clean.contains("while x < 10:"));
        assert!(clean.contains("[trecho de código omitido pelo Mentor]"));
    }

    #[test]
    fn test_wallet_sha256_tamper_detection() {
        let mut cert = MasteryGameMasterAgent::generate_signed_wallet(
            "Aluno Teste",
            "prof_local_01",
            "mod_01_fundamentos",
            5,
            1200,
            "2026-11-14T10:00:00Z",
        );
        assert!(MasteryGameMasterAgent::verify_wallet_signature(&cert));

        // Simula adulteração manual do XP no arquivo exportado
        cert.total_earned_xp = 99999;
        assert!(!MasteryGameMasterAgent::verify_wallet_signature(&cert));
    }

    #[test]
    fn test_streak_anti_cheat_clock_tampering() {
        let (streak, date, tampered) =
            MasteryGameMasterAgent::compute_streak_update(4, Some("2026-11-14"), "2026-11-12");
        assert!(tampered);
        assert_eq!(streak, 4);
        assert_eq!(date, "2026-11-14");
    }
}
