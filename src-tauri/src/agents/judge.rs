use crate::agents::{AstInspectionReport, LevelRuleConfig, TestCaseSpec};
use crate::sandbox::ast_inspector::AstInspector;
use crate::sandbox::runner::{SandboxExecutionSummary, SandboxRunner};

pub struct AstJudgeAgent;

impl AstJudgeAgent {
    /// Analisa a Árvore Sintática Abstrata (AST) do código Python do aluno
    pub fn inspect_code(
        source_code: &str,
        rules: &LevelRuleConfig,
        test_cases: &[TestCaseSpec],
    ) -> AstInspectionReport {
        AstInspector::inspect(source_code, rules, test_cases)
    }

    /// Executa todos os casos de teste públicos e ocultos no Sandbox isolado após aprovação da AST
    pub fn run_test_suite(
        source_code: &str,
        test_cases: &[TestCaseSpec],
        max_time_ms: u64,
    ) -> SandboxExecutionSummary {
        SandboxRunner::execute_test_suite(source_code, test_cases, max_time_ms)
    }
}
