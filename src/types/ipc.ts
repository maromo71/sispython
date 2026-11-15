// Contratos TypeScript espelhados estritamente das structs Rust em src-tauri/src/ipc/commands.rs

export interface ProfileRecord {
  id: string;
  display_name: string;
  theme_preference: 'dark_tech' | 'light_tech';
  total_xp: number;
  current_level_rank: number;
  logic_fragments: number;
  current_streak_days: number;
  last_activity_date: string | null;
}

export interface ModuleRecord {
  id: string;
  order_index: number;
  title: string;
  description: string;
  required_xp_to_unlock: number;
}

export type LevelType = 'STANDARD' | 'FIXATION' | 'BOSS';
export type LevelStatus = 'LOCKED' | 'UNLOCKED' | 'COMPLETED_MASTERY';

export interface LevelProgressRecord {
  id: string;
  module_id: string;
  order_index: number;
  title: string;
  level_type: LevelType;
  problem_statement_md: string;
  starter_code: string;
  must_contain_nodes: string[];
  forbidden_nodes: string[];
  base_xp_reward: number;
  expected_time_seconds: number;
  status: LevelStatus;
  attempts_count: number;
  hints_used_count: number;
  best_execution_time_ms: number | null;
  earned_xp: number;
  needs_fixation_review: boolean;
  last_submitted_code: string | null;
}

export interface PublicTestCaseView {
  id: string;
  input_data_json: string;
  expected_output_json: string;
  failure_concept_tag: string;
}

export interface BadgeStatusRecord {
  id: string;
  code: string;
  title: string;
  description: string;
  icon_name: string;
  unlocked: boolean;
  unlocked_at: string | null;
}

export interface CurriculumPayload {
  profile: ProfileRecord;
  modules: ModuleRecord[];
  levels: LevelProgressRecord[];
  public_test_cases: Record<string, PublicTestCaseView[]>;
  badges: BadgeStatusRecord[];
  ai_key_configured: boolean;
  ai_status_label: string;
}

export interface AstInspectionReport {
  is_valid: boolean;
  detected_nodes: string[];
  missing_required_nodes: string[];
  triggered_forbidden_nodes: string[];
  suspected_hardcode: boolean;
  syntax_error_detail: [string, number] | null;
}

export interface VariableTraceEntry {
  step: number;
  line: number;
  variables: Record<string, string>;
}

export interface TestCaseResult {
  test_case_id: string;
  passed: boolean;
  is_hidden_edge_case: boolean;
  failure_concept_tag: string;
  input_preview: string;
  expected_preview: string | null;
  actual_preview: string | null;
  error_message: string | null;
}

export interface SandboxExecutionSummary {
  all_passed: boolean;
  verdict: 'PASSED' | 'TEST_FAILED' | 'TIMEOUT_LOOP' | 'SYNTAX_ERROR';
  execution_time_ms: number;
  stdout: string;
  stdout_truncated: boolean;
  variable_trace: VariableTraceEntry[];
  test_results: TestCaseResult[];
  first_failed_concept_tag: string | null;
  runtime_error: string | null;
}

export interface SocraticHintResponse {
  diagnostic_category: string;
  friendly_explanation: string;
  socratic_question: string;
  focus_line: number;
  conceptual_reminder: string;
  provider_used: string;
}

export interface MasteryRewardPayload {
  earned_xp: number;
  new_total_xp: number;
  updated_streak_days: number;
  next_level_unlocked_id: string | null;
  scheduled_fixation_review: boolean;
  newly_unlocked_badges: string[];
}

export type SubmissionVerdict =
  | 'PASSED'
  | 'AST_REJECTED'
  | 'TEST_FAILED'
  | 'TIMEOUT_LOOP'
  | 'SYNTAX_ERROR';

export interface SubmitCodeResponse {
  verdict: SubmissionVerdict;
  ast_report: AstInspectionReport;
  execution_summary: SandboxExecutionSummary | null;
  socratic_hint: SocraticHintResponse | null;
  reward: MasteryRewardPayload | null;
  updated_profile: ProfileRecord;
  clock_tamper_detected: boolean;
}

export interface SignedWalletCertificate {
  student_name: string;
  profile_id: string;
  module_id: string;
  completed_levels_count: number;
  total_earned_xp: number;
  issued_at_iso: string;
  sha256_signature: string;
}

export interface ApprovedSubmissionEntry {
  level_id: string;
  level_title: string;
  attempts_count: number;
  hints_used_count: number;
  best_execution_time_ms: number | null;
  earned_xp: number;
  approved_code: string;
  completed_at: string | null;
}

export interface SignedWalletExport {
  certificate: SignedWalletCertificate;
  module_title: string;
  total_attempts: number;
  total_hints_used: number;
  approved_submissions: ApprovedSubmissionEntry[];
  html_report: string;
}

export type ModuleDifficulty = 'INICIANTE' | 'INTERMEDIARIO' | 'AVANCADO';

export interface GenerateAiModuleRequest {
  title: string;
  topic: string;
  difficulty: ModuleDifficulty;
  exercise_count: number;
  custom_instructions: string | null;
}

export interface ExpandModuleExercisesRequest {
  module_id: string;
  target_exercise_count: number;
  focus_topic: string | null;
}

export interface AiModuleGenerationResponse {
  curriculum: CurriculumPayload;
  affected_module_id: string;
  generated_levels_count: number;
  provider_used: string;
}
