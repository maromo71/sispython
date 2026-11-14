use crate::agents::game_master::MasteryGameMasterAgent;
use crate::agents::{LevelRuleConfig, SignedWalletCertificate, TestCaseSpec};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub const INITIAL_MIGRATION_SQL: &str = include_str!("../../migrations/0001_initial_schema.sql");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub id: String,
    pub display_name: String,
    pub theme_preference: String,
    pub total_xp: i32,
    pub current_level_rank: i32,
    pub logic_fragments: i32,
    pub current_streak_days: i32,
    pub last_activity_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub id: String,
    pub order_index: i32,
    pub title: String,
    pub description: String,
    pub required_xp_to_unlock: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelProgressRecord {
    pub id: String,
    pub module_id: String,
    pub order_index: i32,
    pub title: String,
    pub level_type: String,
    pub problem_statement_md: String,
    pub starter_code: String,
    pub must_contain_nodes: Vec<String>,
    pub forbidden_nodes: Vec<String>,
    pub base_xp_reward: i32,
    pub expected_time_seconds: i32,
    pub status: String, // "LOCKED" | "UNLOCKED" | "COMPLETED_MASTERY"
    pub attempts_count: i32,
    pub hints_used_count: i32,
    pub best_execution_time_ms: Option<i32>,
    pub earned_xp: i32,
    pub needs_fixation_review: bool,
    pub last_submitted_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicTestCaseView {
    pub id: String,
    pub input_data_json: String,
    pub expected_output_json: String,
    pub failure_concept_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeStatusRecord {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub icon_name: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedSubmissionEntry {
    pub level_id: String,
    pub level_title: String,
    pub attempts_count: i32,
    pub hints_used_count: i32,
    pub best_execution_time_ms: Option<i32>,
    pub earned_xp: i32,
    pub approved_code: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedWalletExport {
    pub certificate: SignedWalletCertificate,
    pub module_title: String,
    pub total_attempts: i32,
    pub total_hints_used: i32,
    pub approved_submissions: Vec<ApprovedSubmissionEntry>,
    pub html_report: String,
}

pub fn generate_uid(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{}_{}_{}", prefix, nanos, seq)
}

pub struct DatabaseRepository {
    pub pool: SqlitePool,
}

impl DatabaseRepository {
    pub async fn connect_and_migrate(database_url: &str) -> Result<Self, String> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| e.to_string())?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| format!("Falha ao abrir SQLite: {}", e))?;

        sqlx::raw_sql(INITIAL_MIGRATION_SQL)
            .execute(&pool)
            .await
            .map_err(|e| format!("Falha na migração inicial SQLite: {}", e))?;

        Ok(Self { pool })
    }

    pub async fn get_default_profile(&self) -> Result<ProfileRecord, String> {
        let row = sqlx::query(
            "SELECT id, display_name, theme_preference, total_xp, current_level_rank, logic_fragments, current_streak_days, last_activity_date \
             FROM profiles WHERE id = 'prof_local_01' LIMIT 1",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ProfileRecord {
            id: row.get("id"),
            display_name: row.get("display_name"),
            theme_preference: row.get("theme_preference"),
            total_xp: row.get("total_xp"),
            current_level_rank: row.get("current_level_rank"),
            logic_fragments: row.get("logic_fragments"),
            current_streak_days: row.get("current_streak_days"),
            last_activity_date: row.get("last_activity_date"),
        })
    }

    pub async fn get_modules(&self) -> Result<Vec<ModuleRecord>, String> {
        let rows = sqlx::query(
            "SELECT id, order_index, title, description, required_xp_to_unlock FROM modules ORDER BY order_index ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|r| ModuleRecord {
                id: r.get("id"),
                order_index: r.get("order_index"),
                title: r.get("title"),
                description: r.get("description"),
                required_xp_to_unlock: r.get("required_xp_to_unlock"),
            })
            .collect())
    }

    pub async fn get_levels_with_progress(
        &self,
        profile_id: &str,
    ) -> Result<Vec<LevelProgressRecord>, String> {
        let rows = sqlx::query(
            "SELECT l.id, l.module_id, l.order_index, l.title, l.level_type, l.problem_statement_md, \
             l.starter_code, l.ast_must_contain_json, l.ast_forbidden_json, l.base_xp_reward, l.expected_time_seconds, \
             COALESCE(up.status, 'LOCKED') AS status, COALESCE(up.attempts_count, 0) AS attempts_count, \
             COALESCE(up.hints_used_count, 0) AS hints_used_count, up.best_execution_time_ms, \
             COALESCE(up.earned_xp, 0) AS earned_xp, COALESCE(up.needs_fixation_review, 0) AS needs_fixation_review, \
             (SELECT s.submitted_code FROM submissions s WHERE s.profile_id = ?1 AND s.level_id = l.id ORDER BY s.created_at DESC, s.rowid DESC LIMIT 1) AS last_submitted_code \
             FROM levels l \
             LEFT JOIN user_progress up ON up.level_id = l.id AND up.profile_id = ?1 \
             ORDER BY l.order_index ASC",
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let must_json: String = r.get("ast_must_contain_json");
            let forb_json: String = r.get("ast_forbidden_json");
            let must_contain_nodes: Vec<String> =
                serde_json::from_str(&must_json).unwrap_or_default();
            let forbidden_nodes: Vec<String> =
                serde_json::from_str(&forb_json).unwrap_or_default();
            let fixation_int: i32 = r.get("needs_fixation_review");

            out.push(LevelProgressRecord {
                id: r.get("id"),
                module_id: r.get("module_id"),
                order_index: r.get("order_index"),
                title: r.get("title"),
                level_type: r.get("level_type"),
                problem_statement_md: r.get("problem_statement_md"),
                starter_code: r.get("starter_code"),
                must_contain_nodes,
                forbidden_nodes,
                base_xp_reward: r.get("base_xp_reward"),
                expected_time_seconds: r.get("expected_time_seconds"),
                status: r.get("status"),
                attempts_count: r.get("attempts_count"),
                hints_used_count: r.get("hints_used_count"),
                best_execution_time_ms: r.get("best_execution_time_ms"),
                earned_xp: r.get("earned_xp"),
                needs_fixation_review: fixation_int != 0,
                last_submitted_code: r.get("last_submitted_code"),
            });
        }
        Ok(out)
    }

    pub async fn get_public_test_cases_for_all_levels(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<PublicTestCaseView>>, String> {
        let rows = sqlx::query(
            "SELECT id, level_id, input_data_json, expected_output_json, failure_concept_tag \
             FROM test_cases WHERE is_hidden_edge_case = 0 ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut map: std::collections::HashMap<String, Vec<PublicTestCaseView>> =
            std::collections::HashMap::new();
        for r in rows {
            let level_id: String = r.get("level_id");
            map.entry(level_id).or_default().push(PublicTestCaseView {
                id: r.get("id"),
                input_data_json: r.get("input_data_json"),
                expected_output_json: r.get("expected_output_json"),
                failure_concept_tag: r.get("failure_concept_tag"),
            });
        }
        Ok(map)
    }

    pub async fn get_badges_status(&self, profile_id: &str) -> Result<Vec<BadgeStatusRecord>, String> {
        let rows = sqlx::query(
            "SELECT b.id, b.code, b.title, b.description, b.icon_name, ub.unlocked_at \
             FROM badges b \
             LEFT JOIN user_badges ub ON ub.badge_id = b.id AND ub.profile_id = ?1 \
             ORDER BY b.id ASC",
        )
        .bind(profile_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let unlocked_at: Option<String> = r.get("unlocked_at");
                BadgeStatusRecord {
                    id: r.get("id"),
                    code: r.get("code"),
                    title: r.get("title"),
                    description: r.get("description"),
                    icon_name: r.get("icon_name"),
                    unlocked: unlocked_at.is_some(),
                    unlocked_at,
                }
            })
            .collect())
    }

    pub async fn get_level_rules_and_tests(
        &self,
        level_id: &str,
    ) -> Result<(LevelRuleConfig, String, i32, Vec<TestCaseSpec>), String> {
        let lvl_row = sqlx::query(
            "SELECT id, module_id, order_index, level_type, problem_statement_md, ast_must_contain_json, ast_forbidden_json, base_xp_reward, expected_time_seconds \
             FROM levels WHERE id = ?1",
        )
        .bind(level_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Fase '{}' não encontrada.", level_id))?;

        let must_json: String = lvl_row.get("ast_must_contain_json");
        let forb_json: String = lvl_row.get("ast_forbidden_json");

        let rules = LevelRuleConfig {
            level_id: lvl_row.get("id"),
            level_type: lvl_row.get("level_type"),
            problem_statement_md: lvl_row.get("problem_statement_md"),
            must_contain_nodes: serde_json::from_str(&must_json).unwrap_or_default(),
            forbidden_nodes: serde_json::from_str(&forb_json).unwrap_or_default(),
            base_xp_reward: lvl_row.get("base_xp_reward"),
            expected_time_seconds: lvl_row.get("expected_time_seconds"),
        };

        let module_id: String = lvl_row.get("module_id");
        let order_index: i32 = lvl_row.get("order_index");

        let tc_rows = sqlx::query(
            "SELECT id, input_data_json, expected_output_json, is_hidden_edge_case, failure_concept_tag \
             FROM test_cases WHERE level_id = ?1 ORDER BY is_hidden_edge_case ASC, id ASC",
        )
        .bind(level_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let test_cases = tc_rows
            .into_iter()
            .map(|r| {
                let hidden: i32 = r.get("is_hidden_edge_case");
                TestCaseSpec {
                    id: r.get("id"),
                    input_data_json: r.get("input_data_json"),
                    expected_output_json: r.get("expected_output_json"),
                    is_hidden_edge_case: hidden != 0,
                    failure_concept_tag: r.get("failure_concept_tag"),
                }
            })
            .collect();

        Ok((rules, module_id, order_index, test_cases))
    }

    pub async fn get_user_progress_for_level(
        &self,
        profile_id: &str,
        level_id: &str,
    ) -> Result<(String, i32, i32, Option<i32>, i32), String> {
        let row = sqlx::query(
            "SELECT status, attempts_count, hints_used_count, best_execution_time_ms, earned_xp \
             FROM user_progress WHERE profile_id = ?1 AND level_id = ?2",
        )
        .bind(profile_id)
        .bind(level_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        match row {
            Some(r) => Ok((
                r.get("status"),
                r.get("attempts_count"),
                r.get("hints_used_count"),
                r.get("best_execution_time_ms"),
                r.get("earned_xp"),
            )),
            None => Ok(("LOCKED".to_string(), 0, 0, None, 0)),
        }
    }

    pub async fn record_failed_submission(
        &self,
        profile_id: &str,
        level_id: &str,
        submitted_code: &str,
        verdict: &str,
        execution_time_ms: i64,
        variable_trace_json: &str,
        provider_used: &str,
        diagnostic_category: &str,
        socratic_question: &str,
        focus_line: usize,
    ) -> Result<(), String> {
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO user_progress (id, profile_id, level_id, status, attempts_count) \
             VALUES (?1, ?2, ?3, 'UNLOCKED', 1) \
             ON CONFLICT(profile_id, level_id) DO UPDATE SET attempts_count = attempts_count + 1",
        )
        .bind(generate_uid("up"))
        .bind(profile_id)
        .bind(level_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let sub_id = generate_uid("sub");
        sqlx::query(
            "INSERT INTO submissions (id, profile_id, level_id, submitted_code, verdict, execution_time_ms, variable_trace_json) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(&sub_id)
        .bind(profile_id)
        .bind(level_id)
        .bind(submitted_code)
        .bind(verdict)
        .bind(execution_time_ms)
        .bind(variable_trace_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO socratic_interactions (id, submission_id, provider_used, diagnostic_category, socratic_question, focus_line) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(generate_uid("soc"))
        .bind(&sub_id)
        .bind(provider_used)
        .bind(diagnostic_category)
        .bind(socratic_question)
        .bind(focus_line as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn record_passed_submission_atomic(
        &self,
        profile_id: &str,
        module_id: &str,
        level_id: &str,
        order_index: i32,
        level_type: &str,
        submitted_code: &str,
        execution_time_ms: i64,
        variable_trace_json: &str,
        base_xp: i32,
        expected_time_seconds: i32,
        resolution_time_seconds: i32,
        today_iso_date: &str,
    ) -> Result<(crate::agents::MasteryRewardPayload, bool), String> {
        let mut tx = self.pool.begin().await.map_err(|e| e.to_string())?;

        let prof_row = sqlx::query(
            "SELECT total_xp, current_streak_days, last_activity_date FROM profiles WHERE id = ?1",
        )
        .bind(profile_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let current_total_xp: i32 = prof_row.get("total_xp");
        let current_streak: i32 = prof_row.get("current_streak_days");
        let last_activity: Option<String> = prof_row.get("last_activity_date");

        let prog_row = sqlx::query(
            "SELECT status, attempts_count, hints_used_count, best_execution_time_ms, earned_xp \
             FROM user_progress WHERE profile_id = ?1 AND level_id = ?2",
        )
        .bind(profile_id)
        .bind(level_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        let (prev_status, prev_attempts, hints_used, prev_best_ms, prev_earned_xp) = match prog_row {
            Some(r) => (
                r.get::<String, _>("status"),
                r.get::<i32, _>("attempts_count"),
                r.get::<i32, _>("hints_used_count"),
                r.get::<Option<i32>, _>("best_execution_time_ms"),
                r.get::<i32, _>("earned_xp"),
            ),
            None => ("UNLOCKED".to_string(), 0, 0, None, 0),
        };

        let total_attempts = prev_attempts + 1;
        let already_completed = prev_status == "COMPLETED_MASTERY";

        // Busca próxima fase do mesmo módulo para desbloquear (Hard Gatekeeping)
        let next_lvl_row = sqlx::query(
            "SELECT id FROM levels WHERE module_id = ?1 AND order_index = ?2 LIMIT 1",
        )
        .bind(module_id)
        .bind(order_index + 1)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        let next_level_id: Option<String> = next_lvl_row.map(|r| r.get("id"));

        let (updated_streak, new_activity_date, clock_tamper) =
            MasteryGameMasterAgent::compute_streak_update(
                current_streak,
                last_activity.as_deref(),
                today_iso_date,
            );

        let mut reward = MasteryGameMasterAgent::build_mastery_reward(
            if already_completed { 0 } else { base_xp },
            current_total_xp,
            total_attempts,
            hints_used,
            updated_streak,
            resolution_time_seconds,
            expected_time_seconds,
            level_type == "BOSS",
            next_level_id.clone(),
        );
        reward.updated_streak_days = updated_streak;

        // 1. Grava submissão aprovada
        sqlx::query(
            "INSERT INTO submissions (id, profile_id, level_id, submitted_code, verdict, execution_time_ms, variable_trace_json) \
             VALUES (?1, ?2, ?3, ?4, 'PASSED', ?5, ?6)",
        )
        .bind(generate_uid("sub"))
        .bind(profile_id)
        .bind(level_id)
        .bind(submitted_code)
        .bind(execution_time_ms)
        .bind(variable_trace_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // 2. Atualiza progresso da fase para COMPLETED_MASTERY
        let best_ms = match prev_best_ms {
            Some(prev) => prev.min(execution_time_ms as i32),
            None => execution_time_ms as i32,
        };
        let final_level_xp = if already_completed {
            prev_earned_xp
        } else {
            reward.earned_xp
        };
        let fixation_flag = if reward.scheduled_fixation_review { 1 } else { 0 };

        sqlx::query(
            "INSERT INTO user_progress (id, profile_id, level_id, status, attempts_count, hints_used_count, best_execution_time_ms, earned_xp, needs_fixation_review, completed_at) \
             VALUES (?1, ?2, ?3, 'COMPLETED_MASTERY', ?4, ?5, ?6, ?7, ?8, datetime('now')) \
             ON CONFLICT(profile_id, level_id) DO UPDATE SET \
             status = 'COMPLETED_MASTERY', \
             attempts_count = ?4, \
             best_execution_time_ms = ?6, \
             earned_xp = ?7, \
             needs_fixation_review = ?8, \
             completed_at = COALESCE(completed_at, datetime('now'))",
        )
        .bind(generate_uid("up"))
        .bind(profile_id)
        .bind(level_id)
        .bind(total_attempts)
        .bind(hints_used)
        .bind(best_ms)
        .bind(final_level_xp)
        .bind(fixation_flag)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // 3. Desbloqueia próxima fase se existir e estiver LOCKED
        if let Some(ref next_id) = next_level_id {
            sqlx::query(
                "INSERT INTO user_progress (id, profile_id, level_id, status) \
                 VALUES (?1, ?2, ?3, 'UNLOCKED') \
                 ON CONFLICT(profile_id, level_id) DO UPDATE SET \
                 status = CASE WHEN status = 'LOCKED' THEN 'UNLOCKED' ELSE status END",
            )
            .bind(generate_uid("up"))
            .bind(profile_id)
            .bind(next_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        }

        // 4. Atualiza perfil (XP, rank, fragmentos lógicos, streak)
        let new_rank = MasteryGameMasterAgent::compute_level_rank(reward.new_total_xp);
        let fragment_gain = if already_completed { 0 } else { 1 };
        sqlx::query(
            "UPDATE profiles SET total_xp = ?1, current_level_rank = ?2, logic_fragments = logic_fragments + ?3, \
             current_streak_days = ?4, last_activity_date = ?5 WHERE id = ?6",
        )
        .bind(reward.new_total_xp)
        .bind(new_rank)
        .bind(fragment_gain)
        .bind(updated_streak)
        .bind(&new_activity_date)
        .bind(profile_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // 5. Verifica badge AUTONOMOUS_MIND (3 fases concluídas com 0 pistas)
        let zero_hint_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_progress WHERE profile_id = ?1 AND status = 'COMPLETED_MASTERY' AND hints_used_count = 0",
        )
        .bind(profile_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        if zero_hint_count >= 3 && !reward.newly_unlocked_badges.contains(&"AUTONOMOUS_MIND".to_string()) {
            reward.newly_unlocked_badges.push("AUTONOMOUS_MIND".to_string());
        }

        let mut truly_new_badges = Vec::new();
        for badge_code in &reward.newly_unlocked_badges {
            if let Some(badge_id) = sqlx::query_scalar::<_, String>(
                "SELECT id FROM badges WHERE code = ?1 LIMIT 1",
            )
            .bind(badge_code)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
            {
                let res = sqlx::query(
                    "INSERT OR IGNORE INTO user_badges (id, profile_id, badge_id) VALUES (?1, ?2, ?3)",
                )
                .bind(generate_uid("ub"))
                .bind(profile_id)
                .bind(&badge_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
                if res.rows_affected() > 0 {
                    truly_new_badges.push(badge_code.clone());
                }
            }
        }
        reward.newly_unlocked_badges = truly_new_badges;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok((reward, clock_tamper))
    }

    pub async fn increment_hint_usage(&self, profile_id: &str, level_id: &str) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_progress (id, profile_id, level_id, status, hints_used_count) \
             VALUES (?1, ?2, ?3, 'UNLOCKED', 1) \
             ON CONFLICT(profile_id, level_id) DO UPDATE SET hints_used_count = hints_used_count + 1",
        )
        .bind(generate_uid("up"))
        .bind(profile_id)
        .bind(level_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_theme(&self, profile_id: &str, theme: &str) -> Result<(), String> {
        if theme != "dark_tech" && theme != "light_tech" {
            return Err("Tema inválido".to_string());
        }
        sqlx::query("UPDATE profiles SET theme_preference = ?1 WHERE id = ?2")
            .bind(theme)
            .bind(profile_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn update_display_name(&self, profile_id: &str, name: &str) -> Result<(), String> {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.len() > 80 {
            return Err("Nome do estudante deve ter entre 1 e 80 caracteres.".to_string());
        }
        sqlx::query("UPDATE profiles SET display_name = ?1 WHERE id = ?2")
            .bind(trimmed)
            .bind(profile_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn export_signed_wallet(
        &self,
        profile_id: &str,
        module_id: &str,
    ) -> Result<SignedWalletExport, String> {
        let profile = self.get_default_profile().await?;
        let module_title: String = sqlx::query_scalar(
            "SELECT title FROM modules WHERE id = ?1 LIMIT 1",
        )
        .bind(module_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Módulo não encontrado.".to_string())?;

        let rows = sqlx::query(
            "SELECT l.id AS level_id, l.title AS level_title, up.attempts_count, up.hints_used_count, \
             up.best_execution_time_ms, up.earned_xp, up.completed_at, \
             (SELECT s.submitted_code FROM submissions s WHERE s.profile_id = ?1 AND s.level_id = l.id AND s.verdict = 'PASSED' ORDER BY s.created_at DESC, s.rowid DESC LIMIT 1) AS approved_code \
             FROM levels l \
             INNER JOIN user_progress up ON up.level_id = l.id AND up.profile_id = ?1 \
             WHERE l.module_id = ?2 AND up.status = 'COMPLETED_MASTERY' \
             ORDER BY l.order_index ASC",
        )
        .bind(profile_id)
        .bind(module_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut approved_submissions = Vec::new();
        let mut total_attempts = 0;
        let mut total_hints_used = 0;
        let mut module_xp = 0;

        for r in rows {
            let attempts: i32 = r.get("attempts_count");
            let hints: i32 = r.get("hints_used_count");
            let earned: i32 = r.get("earned_xp");
            total_attempts += attempts;
            total_hints_used += hints;
            module_xp += earned;

            approved_submissions.push(ApprovedSubmissionEntry {
                level_id: r.get("level_id"),
                level_title: r.get("level_title"),
                attempts_count: attempts,
                hints_used_count: hints,
                best_execution_time_ms: r.get("best_execution_time_ms"),
                earned_xp: earned,
                approved_code: r.get::<Option<String>, _>("approved_code").unwrap_or_default(),
                completed_at: r.get("completed_at"),
            });
        }

        let issued_at_iso = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let cert = MasteryGameMasterAgent::generate_signed_wallet(
            &profile.display_name,
            &profile.id,
            module_id,
            approved_submissions.len(),
            module_xp,
            &issued_at_iso,
        );

        let html_report = Self::render_html_certificate(&cert, &module_title, &approved_submissions);
        let payload_json = serde_json::to_string(&cert).unwrap_or_default();

        sqlx::query(
            "INSERT INTO signed_certificates (id, profile_id, module_id, payload_json, sha256_signature, issued_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(generate_uid("cert"))
        .bind(profile_id)
        .bind(module_id)
        .bind(&payload_json)
        .bind(&cert.sha256_signature)
        .bind(&cert.issued_at_iso)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(SignedWalletExport {
            certificate: cert,
            module_title,
            total_attempts,
            total_hints_used,
            approved_submissions,
            html_report,
        })
    }

    fn render_html_certificate(
        cert: &SignedWalletCertificate,
        module_title: &str,
        entries: &[ApprovedSubmissionEntry],
    ) -> String {
        let mut rows_html = String::new();
        for e in entries {
            let escaped_code = e
                .approved_code
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            rows_html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{} ms</td><td>{} XP</td></tr><tr><td colspan=\"5\"><pre><code>{}</code></pre></td></tr>",
                e.level_title,
                e.attempts_count,
                e.hints_used_count,
                e.best_execution_time_ms.unwrap_or(0),
                e.earned_xp,
                escaped_code
            ));
        }

        format!(
            "<!DOCTYPE html><html lang=\"pt-BR\"><head><meta charset=\"UTF-8\"><title>Carteira LogicKiosk - {}</title>\
            <style>body{{font-family:Inter,system-ui,sans-serif;background:#0B0F19;color:#F8FAFC;padding:32px;max-width:920px;margin:auto;}}\
            .card{{background:#111827;border:1px solid #334155;border-radius:12px;padding:24px;margin-bottom:20px;}}\
            .sig{{font-family:monospace;background:#1E293B;padding:12px;border-radius:8px;word-break:break-all;color:#06B6D4;}}\
            table{{width:100%;border-collapse:collapse;margin-top:16px;}}th,td{{border-bottom:1px solid #334155;padding:10px;text-align:left;}}\
            pre{{background:#0B0F19;padding:12px;border-radius:8px;overflow-x:auto;font-size:13px;color:#10B981;}}</style></head>\
            <body><div class=\"card\"><h1>Carteira de Progresso Acadêmico — LogicKiosk</h1>\
            <p><strong>Estudante:</strong> {} ({})</p>\
            <p><strong>Módulo:</strong> {}</p>\
            <p><strong>Fases com Maestria:</strong> {} | <strong>XP Total do Módulo:</strong> {} XP</p>\
            <p><strong>Emitido em:</strong> {}</p>\
            <p><strong>Assinatura SHA-256:</strong></p><div class=\"sig\">{}</div></div>\
            <div class=\"card\"><h2>Histórico de Códigos Aprovados</h2><table><thead><tr><th>Fase</th><th>Tentativas</th><th>Pistas</th><th>Melhor Tempo</th><th>XP</th></tr></thead><tbody>{}</tbody></table></div></body></html>",
            cert.student_name,
            cert.student_name,
            cert.profile_id,
            module_title,
            cert.completed_levels_count,
            cert.total_earned_xp,
            cert.issued_at_iso,
            cert.sha256_signature,
            rows_html
        )
    }
}
