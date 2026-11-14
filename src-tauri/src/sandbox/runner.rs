use crate::agents::TestCaseSpec;
use rustpython_parser::{ast, Parse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

pub const DEFAULT_MAX_EXECUTION_TIME_MS: u64 = 2000;
pub const MAX_OUTPUT_CHARS: usize = 10_000;
pub const MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_COLLECTION_LEN: usize = 100_000;
pub const MAX_RECURSION_DEPTH: usize = 64;
pub const MAX_TRACE_ENTRIES: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTraceEntry {
    pub step: usize,
    pub line: usize,
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub test_case_id: String,
    pub passed: bool,
    pub is_hidden_edge_case: bool,
    pub failure_concept_tag: String,
    pub input_preview: String,
    pub expected_preview: Option<String>,
    pub actual_preview: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionSummary {
    pub all_passed: bool,
    pub verdict: String, // "PASSED" | "TEST_FAILED" | "TIMEOUT_LOOP" | "SYNTAX_ERROR"
    pub execution_time_ms: i64,
    pub stdout: String,
    pub stdout_truncated: bool,
    pub variable_trace: Vec<VariableTraceEntry>,
    pub test_results: Vec<TestCaseResult>,
    pub first_failed_concept_tag: Option<String>,
    pub runtime_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PyVal {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<PyVal>),
    Tuple(Vec<PyVal>),
    Dict(Vec<(PyVal, PyVal)>),
}

impl PyVal {
    pub fn from_json(v: &Value) -> Result<Self, String> {
        match v {
            Value::Null => Ok(PyVal::None),
            Value::Bool(b) => Ok(PyVal::Bool(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(PyVal::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(PyVal::Float(f))
                } else {
                    Err("Número inválido no JSON".to_string())
                }
            }
            Value::String(s) => Ok(PyVal::Str(s.clone())),
            Value::Array(arr) => {
                let mut items = Vec::with_capacity(arr.len());
                for el in arr {
                    items.push(PyVal::from_json(el)?);
                }
                Ok(PyVal::List(items))
            }
            Value::Object(map) => {
                let mut pairs = Vec::with_capacity(map.len());
                for (k, val) in map {
                    pairs.push((PyVal::Str(k.clone()), PyVal::from_json(val)?));
                }
                Ok(PyVal::Dict(pairs))
            }
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            PyVal::None => Value::Null,
            PyVal::Bool(b) => Value::Bool(*b),
            PyVal::Int(i) => Value::Number((*i).into()),
            PyVal::Float(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            PyVal::Str(s) => Value::String(s.clone()),
            PyVal::List(items) | PyVal::Tuple(items) => {
                Value::Array(items.iter().map(|i| i.to_json()).collect())
            }
            PyVal::Dict(pairs) => {
                let mut map = serde_json::Map::new();
                for (k, v) in pairs {
                    map.insert(k.to_display_string(), v.to_json());
                }
                Value::Object(map)
            }
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            PyVal::None => "None".to_string(),
            PyVal::Bool(true) => "True".to_string(),
            PyVal::Bool(false) => "False".to_string(),
            PyVal::Int(i) => i.to_string(),
            PyVal::Float(f) => {
                if (f - f.round()).abs() < 1e-12 && f.abs() < 1e15 {
                    format!("{:.1}", f)
                } else {
                    format!("{}", f)
                }
            }
            PyVal::Str(s) => s.clone(),
            PyVal::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_repr_string()).collect();
                format!("[{}]", inner.join(", "))
            }
            PyVal::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_repr_string()).collect();
                if items.len() == 1 {
                    format!("({},)", inner[0])
                } else {
                    format!("({})", inner.join(", "))
                }
            }
            PyVal::Dict(pairs) => {
                let inner: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_repr_string(), v.to_repr_string()))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
    }

    pub fn to_repr_string(&self) -> String {
        match self {
            PyVal::Str(s) => format!("\"{}\"", s),
            _ => self.to_display_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            PyVal::None => false,
            PyVal::Bool(b) => *b,
            PyVal::Int(i) => *i != 0,
            PyVal::Float(f) => *f != 0.0,
            PyVal::Str(s) => !s.is_empty(),
            PyVal::List(l) | PyVal::Tuple(l) => !l.is_empty(),
            PyVal::Dict(d) => !d.is_empty(),
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            PyVal::Int(i) => Some(*i as f64),
            PyVal::Float(f) => Some(*f),
            PyVal::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn py_eq(&self, other: &PyVal) -> bool {
        match (self, other) {
            (PyVal::None, PyVal::None) => true,
            (PyVal::Bool(a), PyVal::Bool(b)) => a == b,
            (PyVal::Int(a), PyVal::Int(b)) => a == b,
            (PyVal::Float(a), PyVal::Float(b)) => (a - b).abs() < 1e-9,
            (PyVal::Int(a), PyVal::Float(b)) | (PyVal::Float(b), PyVal::Int(a)) => {
                ((*a as f64) - b).abs() < 1e-9
            }
            (PyVal::Str(a), PyVal::Str(b)) => a == b,
            (PyVal::List(a), PyVal::List(b)) | (PyVal::Tuple(a), PyVal::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.py_eq(y))
            }
            (PyVal::Dict(a), PyVal::Dict(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().all(|(ka, va)| {
                    b.iter()
                        .find(|(kb, _)| ka.py_eq(kb))
                        .map(|(_, vb)| va.py_eq(vb))
                        .unwrap_or(false)
                })
            }
            _ => false,
        }
    }

    fn approx_bytes(&self) -> usize {
        match self {
            PyVal::None | PyVal::Bool(_) | PyVal::Int(_) | PyVal::Float(_) => 16,
            PyVal::Str(s) => 24 + s.len(),
            PyVal::List(items) | PyVal::Tuple(items) => {
                24 + items.iter().map(|v| v.approx_bytes()).sum::<usize>()
            }
            PyVal::Dict(pairs) => {
                32 + pairs
                    .iter()
                    .map(|(k, v)| k.approx_bytes() + v.approx_bytes())
                    .sum::<usize>()
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ExecFlow {
    Continue,
    Return(PyVal),
    Break,
    ContinueLoop,
}

#[derive(Debug, Clone)]
enum SandboxError {
    Timeout,
    MemoryLimit(String),
    Runtime(String),
}

#[derive(Debug, Clone)]
struct FnDef {
    params: Vec<String>,
    defaults: Vec<Option<ast::Expr>>,
    body: Vec<ast::Stmt>,
}

struct VmState {
    functions: HashMap<String, FnDef>,
    globals: BTreeMap<String, PyVal>,
    stdout: String,
    stdout_truncated: bool,
    trace: Vec<VariableTraceEntry>,
    step_counter: usize,
    tick_counter: usize,
    recursion_depth: usize,
    allocated_bytes: usize,
    start_time: Instant,
    max_time_ms: u64,
    line_starts: Vec<usize>,
}

impl VmState {
    fn new(source_code: &str, max_time_ms: u64) -> Self {
        let mut line_starts = vec![0];
        for (idx, b) in source_code.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self {
            functions: HashMap::new(),
            globals: BTreeMap::new(),
            stdout: String::new(),
            stdout_truncated: false,
            trace: Vec::new(),
            step_counter: 0,
            tick_counter: 0,
            recursion_depth: 0,
            allocated_bytes: 0,
            start_time: Instant::now(),
            max_time_ms,
            line_starts,
        }
    }

    fn offset_to_line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx.max(1),
        }
    }

    fn check_limits(&mut self) -> Result<(), SandboxError> {
        self.tick_counter += 1;
        // Verifica relógio a cada 64 ticks ou sempre para precisão sub-ms
        if self.tick_counter.is_multiple_of(32)
            && self.start_time.elapsed().as_millis() as u64 >= self.max_time_ms
        {
            return Err(SandboxError::Timeout);
        }
        if self.allocated_bytes > MAX_MEMORY_BYTES {
            return Err(SandboxError::MemoryLimit(
                "MemoryLimitError: limite de 64 MB excedido".to_string(),
            ));
        }
        Ok(())
    }

    fn track_alloc(&mut self, bytes: usize) -> Result<(), SandboxError> {
        self.allocated_bytes = self.allocated_bytes.saturating_add(bytes);
        if self.allocated_bytes > MAX_MEMORY_BYTES {
            return Err(SandboxError::MemoryLimit(
                "MemoryLimitError: alocação excedeu limite de 64 MB".to_string(),
            ));
        }
        Ok(())
    }

    fn push_stdout(&mut self, text: &str) {
        if self.stdout_truncated {
            return;
        }
        let current_len = self.stdout.chars().count();
        if current_len >= MAX_OUTPUT_CHARS {
            self.stdout_truncated = true;
            return;
        }
        let remaining = MAX_OUTPUT_CHARS - current_len;
        let to_add_chars: usize = text.chars().count();
        if to_add_chars <= remaining {
            self.stdout.push_str(text);
        } else {
            let truncated: String = text.chars().take(remaining).collect();
            self.stdout.push_str(&truncated);
            self.stdout_truncated = true;
        }
    }

    fn record_trace(&mut self, line: usize, locals: &BTreeMap<String, PyVal>) {
        self.step_counter += 1;
        if locals.is_empty() && self.globals.is_empty() {
            return;
        }
        let mut snapshot = BTreeMap::new();
        for (k, v) in &self.globals {
            snapshot.insert(k.clone(), v.to_repr_string());
        }
        for (k, v) in locals {
            snapshot.insert(k.clone(), v.to_repr_string());
        }
        let entry = VariableTraceEntry {
            step: self.step_counter,
            line,
            variables: snapshot,
        };
        if self.trace.len() >= MAX_TRACE_ENTRIES {
            // Mantém janela deslizante com as últimas 50 iterações antes de eventual timeout
            self.trace.remove(0);
        }
        self.trace.push(entry);
    }
}

pub struct SandboxRunner;

impl SandboxRunner {
    pub fn execute_test_suite(
        source_code: &str,
        test_cases: &[TestCaseSpec],
        max_time_ms: u64,
    ) -> SandboxExecutionSummary {
        let start = Instant::now();
        let suite = match ast::Suite::parse(source_code, "<kiosk_submission>") {
            Ok(s) => s,
            Err(e) => {
                return SandboxExecutionSummary {
                    all_passed: false,
                    verdict: "SYNTAX_ERROR".to_string(),
                    execution_time_ms: start.elapsed().as_millis() as i64,
                    stdout: String::new(),
                    stdout_truncated: false,
                    variable_trace: vec![],
                    test_results: vec![],
                    first_failed_concept_tag: None,
                    runtime_error: Some(format!("SyntaxError: {}", e.error)),
                };
            }
        };

        let mut vm = VmState::new(source_code, max_time_ms);
        let mut top_locals = BTreeMap::new();
        let mut ordered_fn_names = Vec::new();

        for stmt in &suite {
            if let ast::Stmt::FunctionDef(f) = stmt {
                ordered_fn_names.push(f.name.to_string());
            }
        }

        // Executa definições de nível superior (def, constantes)
        for stmt in &suite {
            match Self::exec_stmt(&mut vm, stmt, &mut top_locals, false) {
                Ok(_) => {}
                Err(SandboxError::Timeout) => {
                    return SandboxExecutionSummary {
                        all_passed: false,
                        verdict: "TIMEOUT_LOOP".to_string(),
                        execution_time_ms: start.elapsed().as_millis() as i64,
                        stdout: vm.stdout,
                        stdout_truncated: vm.stdout_truncated,
                        variable_trace: vm.trace,
                        test_results: vec![],
                        first_failed_concept_tag: Some("INFINITE_LOOP".to_string()),
                        runtime_error: Some(format!(
                            "TimeoutError: execução interrompida ao atingir {}ms",
                            max_time_ms
                        )),
                    };
                }
                Err(SandboxError::MemoryLimit(msg)) | Err(SandboxError::Runtime(msg)) => {
                    return SandboxExecutionSummary {
                        all_passed: false,
                        verdict: "TEST_FAILED".to_string(),
                        execution_time_ms: start.elapsed().as_millis() as i64,
                        stdout: vm.stdout,
                        stdout_truncated: vm.stdout_truncated,
                        variable_trace: vm.trace,
                        test_results: vec![],
                        first_failed_concept_tag: Some("RUNTIME_ERROR".to_string()),
                        runtime_error: Some(msg),
                    };
                }
            }
        }
        vm.globals = top_locals;

        if test_cases.is_empty() {
            return SandboxExecutionSummary {
                all_passed: true,
                verdict: "PASSED".to_string(),
                execution_time_ms: start.elapsed().as_millis() as i64,
                stdout: vm.stdout,
                stdout_truncated: vm.stdout_truncated,
                variable_trace: vm.trace,
                test_results: vec![],
                first_failed_concept_tag: None,
                runtime_error: None,
            };
        }

        let target_fn_name = ordered_fn_names.first().cloned();
        let mut results = Vec::with_capacity(test_cases.len());
        let mut first_failed_tag: Option<String> = None;
        let mut captured_runtime_err: Option<String> = None;
        let mut last_trace = vm.trace.clone();

        for (tc_idx, tc) in test_cases.iter().enumerate() {
            let input_val: Value =
                serde_json::from_str(&tc.input_data_json).unwrap_or(Value::Array(vec![]));
            let expected_val: Value =
                serde_json::from_str(&tc.expected_output_json).unwrap_or(Value::Null);
            let expected_py = PyVal::from_json(&expected_val).unwrap_or(PyVal::None);

            let args_vec: Vec<PyVal> = match &input_val {
                Value::Array(arr) => arr
                    .iter()
                    .map(|v| PyVal::from_json(v).unwrap_or(PyVal::None))
                    .collect(),
                other => vec![PyVal::from_json(other).unwrap_or(PyVal::None)],
            };

            let Some(fn_name) = &target_fn_name else {
                let err = "NameError: nenhuma função definida no código.".to_string();
                if first_failed_tag.is_none() {
                    first_failed_tag = Some(tc.failure_concept_tag.clone());
                    captured_runtime_err = Some(err.clone());
                }
                results.push(TestCaseResult {
                    test_case_id: tc.id.clone(),
                    passed: false,
                    is_hidden_edge_case: tc.is_hidden_edge_case,
                    failure_concept_tag: tc.failure_concept_tag.clone(),
                    input_preview: if tc.is_hidden_edge_case {
                        "[Caso Oculto de Borda]".to_string()
                    } else {
                        tc.input_data_json.clone()
                    },
                    expected_preview: if tc.is_hidden_edge_case {
                        None
                    } else {
                        Some(tc.expected_output_json.clone())
                    },
                    actual_preview: None,
                    error_message: Some(err),
                });
                continue;
            };

            // Reseta traço por caso de teste mantendo trace do último teste (ou do primeiro que falhar/rodar)
            if tc_idx == 0 {
                vm.trace.clear();
                vm.step_counter = 0;
            }
            vm.allocated_bytes = 0;

            match Self::call_user_fn(&mut vm, fn_name, args_vec) {
                Ok(actual_py) => {
                    if tc_idx == 0 || !actual_py.py_eq(&expected_py) {
                        last_trace = vm.trace.clone();
                    }
                    let passed = actual_py.py_eq(&expected_py);
                    if !passed && first_failed_tag.is_none() {
                        first_failed_tag = Some(tc.failure_concept_tag.clone());
                    }
                    results.push(TestCaseResult {
                        test_case_id: tc.id.clone(),
                        passed,
                        is_hidden_edge_case: tc.is_hidden_edge_case,
                        failure_concept_tag: tc.failure_concept_tag.clone(),
                        input_preview: if tc.is_hidden_edge_case {
                            format!("Caso de borda: {}", Self::humanize_concept_tag(&tc.failure_concept_tag))
                        } else {
                            tc.input_data_json.clone()
                        },
                        expected_preview: if tc.is_hidden_edge_case {
                            None // Regra: nunca revelar saída exata de caso oculto
                        } else {
                            Some(expected_py.to_repr_string())
                        },
                        actual_preview: if tc.is_hidden_edge_case {
                            None
                        } else {
                            Some(actual_py.to_repr_string())
                        },
                        error_message: if passed {
                            None
                        } else if tc.is_hidden_edge_case {
                            Some(format!(
                                "Falha ao validar caso de borda: {}",
                                Self::humanize_concept_tag(&tc.failure_concept_tag)
                            ))
                        } else {
                            Some("Saída divergente do esperado.".to_string())
                        },
                    });
                }
                Err(SandboxError::Timeout) => {
                    return SandboxExecutionSummary {
                        all_passed: false,
                        verdict: "TIMEOUT_LOOP".to_string(),
                        execution_time_ms: start.elapsed().as_millis() as i64,
                        stdout: vm.stdout,
                        stdout_truncated: vm.stdout_truncated,
                        variable_trace: vm.trace,
                        test_results: results,
                        first_failed_concept_tag: Some("INFINITE_LOOP".to_string()),
                        runtime_error: Some(format!(
                            "TimeoutError: execução interrompida ao atingir {}ms",
                            max_time_ms
                        )),
                    };
                }
                Err(SandboxError::MemoryLimit(msg)) | Err(SandboxError::Runtime(msg)) => {
                    last_trace = vm.trace.clone();
                    if first_failed_tag.is_none() {
                        first_failed_tag = Some(tc.failure_concept_tag.clone());
                        captured_runtime_err = Some(msg.clone());
                    }
                    results.push(TestCaseResult {
                        test_case_id: tc.id.clone(),
                        passed: false,
                        is_hidden_edge_case: tc.is_hidden_edge_case,
                        failure_concept_tag: tc.failure_concept_tag.clone(),
                        input_preview: if tc.is_hidden_edge_case {
                            format!("Caso de borda: {}", Self::humanize_concept_tag(&tc.failure_concept_tag))
                        } else {
                            tc.input_data_json.clone()
                        },
                        expected_preview: if tc.is_hidden_edge_case {
                            None
                        } else {
                            Some(expected_py.to_repr_string())
                        },
                        actual_preview: None,
                        error_message: Some(msg),
                    });
                }
            }
        }

        let all_passed = results.iter().all(|r| r.passed);
        SandboxExecutionSummary {
            all_passed,
            verdict: if all_passed {
                "PASSED".to_string()
            } else {
                "TEST_FAILED".to_string()
            },
            execution_time_ms: start.elapsed().as_millis() as i64,
            stdout: vm.stdout,
            stdout_truncated: vm.stdout_truncated,
            variable_trace: last_trace,
            test_results: results,
            first_failed_concept_tag: first_failed_tag,
            runtime_error: captured_runtime_err,
        }
    }

    fn humanize_concept_tag(tag: &str) -> &'static str {
        match tag {
            "NEGATIVE_NET_BALANCE" => "números negativos ou saldo negativo",
            "EXACT_BOUNDARY_INCLUSION_195" => "limite inferior exato (195)",
            "EXACT_BOUNDARY_INCLUSION_240" => "limite superior exato (240)",
            "HIGH_BOUNDARY" => "valores acima do limite superior",
            "ALREADY_AT_OR_ABOVE_TARGET" => "condição inicial já igual ou acima da meta",
            "ALL_INVALID_RETURNS_EMPTY_LIST" => "lista sem valores positivos (apenas negativos/zero)",
            "IMMEDIATE_SENTINEL_AT_INDEX_ZERO" => "sentinela logo na primeira posição",
            _ => "cenário de borda oculto",
        }
    }

    fn exec_block(
        vm: &mut VmState,
        stmts: &[ast::Stmt],
        locals: &mut BTreeMap<String, PyVal>,
        in_function: bool,
    ) -> Result<ExecFlow, SandboxError> {
        for stmt in stmts {
            vm.check_limits()?;
            match Self::exec_stmt(vm, stmt, locals, in_function)? {
                ExecFlow::Continue => {}
                other => return Ok(other),
            }
        }
        Ok(ExecFlow::Continue)
    }

    fn exec_stmt(
        vm: &mut VmState,
        stmt: &ast::Stmt,
        locals: &mut BTreeMap<String, PyVal>,
        in_function: bool,
    ) -> Result<ExecFlow, SandboxError> {
        vm.check_limits()?;
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                let mut params = Vec::new();
                for arg in &f.args.posonlyargs {
                    params.push(arg.def.arg.to_string());
                }
                for arg in &f.args.args {
                    params.push(arg.def.arg.to_string());
                }
                let num_defaults = f.args.args.iter().filter(|a| a.default.is_some()).count()
                    + f.args
                        .posonlyargs
                        .iter()
                        .filter(|a| a.default.is_some())
                        .count();
                let mut defaults: Vec<Option<ast::Expr>> = vec![None; params.len()];
                if num_defaults > 0 {
                    let mut all_args = Vec::new();
                    all_args.extend(f.args.posonlyargs.iter());
                    all_args.extend(f.args.args.iter());
                    for (idx, a) in all_args.iter().enumerate() {
                        defaults[idx] = a.default.as_deref().cloned();
                    }
                }
                vm.functions.insert(
                    f.name.to_string(),
                    FnDef {
                        params,
                        defaults,
                        body: f.body.clone(),
                    },
                );
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::Return(r) => {
                let line = vm.offset_to_line(usize::from(r.range.start()));
                let val = match &r.value {
                    Some(expr) => Self::eval_expr(vm, expr, locals)?,
                    None => PyVal::None,
                };
                if in_function {
                    vm.record_trace(line, locals);
                }
                Ok(ExecFlow::Return(val))
            }
            ast::Stmt::Assign(a) => {
                let line = vm.offset_to_line(usize::from(a.range.start()));
                let val = Self::eval_expr(vm, &a.value, locals)?;
                vm.track_alloc(val.approx_bytes())?;
                for target in &a.targets {
                    Self::assign_target(vm, target, val.clone(), locals)?;
                }
                if in_function {
                    vm.record_trace(line, locals);
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::AugAssign(aa) => {
                let line = vm.offset_to_line(usize::from(aa.range.start()));
                let rhs = Self::eval_expr(vm, &aa.value, locals)?;
                let current = Self::eval_expr(vm, &aa.target, locals)?;
                let updated = Self::apply_binop(vm, &aa.op, current, rhs)?;
                Self::assign_target(vm, &aa.target, updated, locals)?;
                if in_function {
                    vm.record_trace(line, locals);
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::AnnAssign(aa) => {
                let line = vm.offset_to_line(usize::from(aa.range.start()));
                if let Some(v) = &aa.value {
                    let val = Self::eval_expr(vm, v, locals)?;
                    Self::assign_target(vm, &aa.target, val, locals)?;
                    if in_function {
                        vm.record_trace(line, locals);
                    }
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::If(i) => {
                let cond = Self::eval_expr(vm, &i.test, locals)?;
                if cond.is_truthy() {
                    Self::exec_block(vm, &i.body, locals, in_function)
                } else {
                    Self::exec_block(vm, &i.orelse, locals, in_function)
                }
            }
            ast::Stmt::While(w) => {
                let line = vm.offset_to_line(usize::from(w.range.start()));
                let mut broke = false;
                while Self::eval_expr(vm, &w.test, locals)?.is_truthy() {
                    vm.check_limits()?;
                    if in_function {
                        vm.record_trace(line, locals);
                    }
                    match Self::exec_block(vm, &w.body, locals, in_function)? {
                        ExecFlow::Continue | ExecFlow::ContinueLoop => {}
                        ExecFlow::Break => {
                            broke = true;
                            break;
                        }
                        ret @ ExecFlow::Return(_) => return Ok(ret),
                    }
                }
                if !broke {
                    if let ret @ ExecFlow::Return(_) =
                        Self::exec_block(vm, &w.orelse, locals, in_function)?
                    {
                        return Ok(ret);
                    }
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::For(f) => {
                let line = vm.offset_to_line(usize::from(f.range.start()));
                let iterable = Self::eval_expr(vm, &f.iter, locals)?;
                let items = Self::to_iterable(&iterable)?;
                let mut broke = false;
                for item in items {
                    vm.check_limits()?;
                    Self::assign_target(vm, &f.target, item, locals)?;
                    if in_function {
                        vm.record_trace(line, locals);
                    }
                    match Self::exec_block(vm, &f.body, locals, in_function)? {
                        ExecFlow::Continue | ExecFlow::ContinueLoop => {}
                        ExecFlow::Break => {
                            broke = true;
                            break;
                        }
                        ret @ ExecFlow::Return(_) => return Ok(ret),
                    }
                }
                if !broke {
                    if let ret @ ExecFlow::Return(_) =
                        Self::exec_block(vm, &f.orelse, locals, in_function)?
                    {
                        return Ok(ret);
                    }
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::Break(_) => Ok(ExecFlow::Break),
            ast::Stmt::Continue(_) => Ok(ExecFlow::ContinueLoop),
            ast::Stmt::Pass(_) => Ok(ExecFlow::Continue),
            ast::Stmt::Expr(e) => {
                let line = vm.offset_to_line(usize::from(e.range.start()));
                Self::eval_expr(vm, &e.value, locals)?;
                if in_function {
                    vm.record_trace(line, locals);
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::Assert(a) => {
                let cond = Self::eval_expr(vm, &a.test, locals)?;
                if !cond.is_truthy() {
                    return Err(SandboxError::Runtime("AssertionError".to_string()));
                }
                Ok(ExecFlow::Continue)
            }
            ast::Stmt::Import(_) | ast::Stmt::ImportFrom(_) => Err(SandboxError::Runtime(
                "SecurityError: imports não são permitidos no sandbox.".to_string(),
            )),
            _ => Ok(ExecFlow::Continue),
        }
    }

    fn assign_target(
        vm: &mut VmState,
        target: &ast::Expr,
        val: PyVal,
        locals: &mut BTreeMap<String, PyVal>,
    ) -> Result<(), SandboxError> {
        match target {
            ast::Expr::Name(n) => {
                locals.insert(n.id.to_string(), val);
                Ok(())
            }
            ast::Expr::Tuple(ast::ExprTuple { elts, .. })
            | ast::Expr::List(ast::ExprList { elts, .. }) => {
                let items = Self::to_iterable(&val)?;
                if items.len() != elts.len() {
                    return Err(SandboxError::Runtime(
                        "ValueError: número incorreto de valores para desempacotar".to_string(),
                    ));
                }
                for (sub_target, item) in elts.iter().zip(items.into_iter()) {
                    Self::assign_target(vm, sub_target, item, locals)?;
                }
                Ok(())
            }
            ast::Expr::Subscript(s) => {
                let idx_val = Self::eval_expr(vm, &s.slice, locals)?;
                if let ast::Expr::Name(n) = s.value.as_ref() {
                    let name = n.id.to_string();
                    let container = locals
                        .get_mut(&name)
                        .ok_or_else(|| SandboxError::Runtime(format!("NameError: '{}'", name)))?;
                    match container {
                        PyVal::List(list) => {
                            let idx = Self::resolve_index(&idx_val, list.len())?;
                            list[idx] = val;
                            Ok(())
                        }
                        PyVal::Dict(pairs) => {
                            if let Some((_, existing)) =
                                pairs.iter_mut().find(|(k, _)| k.py_eq(&idx_val))
                            {
                                *existing = val;
                            } else {
                                pairs.push((idx_val, val));
                            }
                            Ok(())
                        }
                        _ => Err(SandboxError::Runtime(
                            "TypeError: objeto não suporta atribuição por índice".to_string(),
                        )),
                    }
                } else {
                    Err(SandboxError::Runtime(
                        "TypeError: alvo de atribuição não suportado".to_string(),
                    ))
                }
            }
            _ => Err(SandboxError::Runtime(
                "SyntaxError: alvo de atribuição inválido".to_string(),
            )),
        }
    }

    fn resolve_index(idx_val: &PyVal, len: usize) -> Result<usize, SandboxError> {
        let raw = match idx_val {
            PyVal::Int(i) => *i,
            _ => {
                return Err(SandboxError::Runtime(
                    "TypeError: índice de lista deve ser inteiro".to_string(),
                ))
            }
        };
        let resolved = if raw < 0 { len as i64 + raw } else { raw };
        if resolved < 0 || resolved as usize >= len {
            return Err(SandboxError::Runtime(
                "IndexError: list index out of range".to_string(),
            ));
        }
        Ok(resolved as usize)
    }

    fn to_iterable(val: &PyVal) -> Result<Vec<PyVal>, SandboxError> {
        match val {
            PyVal::List(l) | PyVal::Tuple(l) => Ok(l.clone()),
            PyVal::Str(s) => Ok(s.chars().map(|c| PyVal::Str(c.to_string())).collect()),
            PyVal::Dict(pairs) => Ok(pairs.iter().map(|(k, _)| k.clone()).collect()),
            _ => Err(SandboxError::Runtime(
                "TypeError: objeto não é iterável".to_string(),
            )),
        }
    }

    fn eval_expr(
        vm: &mut VmState,
        expr: &ast::Expr,
        locals: &mut BTreeMap<String, PyVal>,
    ) -> Result<PyVal, SandboxError> {
        vm.check_limits()?;
        match expr {
            ast::Expr::Constant(c) => Ok(Self::eval_constant(&c.value)?),
            ast::Expr::Name(n) => {
                let name = n.id.as_str();
                if let Some(v) = locals.get(name) {
                    Ok(v.clone())
                } else if let Some(v) = vm.globals.get(name) {
                    Ok(v.clone())
                } else {
                    Err(SandboxError::Runtime(format!(
                        "NameError: name '{}' is not defined",
                        name
                    )))
                }
            }
            ast::Expr::BinOp(b) => {
                let left = Self::eval_expr(vm, &b.left, locals)?;
                let right = Self::eval_expr(vm, &b.right, locals)?;
                Self::apply_binop(vm, &b.op, left, right)
            }
            ast::Expr::UnaryOp(u) => {
                let val = Self::eval_expr(vm, &u.operand, locals)?;
                match u.op {
                    ast::UnaryOp::USub => match val {
                        PyVal::Int(i) => Ok(PyVal::Int(i.saturating_neg())),
                        PyVal::Float(f) => Ok(PyVal::Float(-f)),
                        PyVal::Bool(b) => Ok(PyVal::Int(if b { -1 } else { 0 })),
                        _ => Err(SandboxError::Runtime("TypeError: operador unário '-' inválido".to_string())),
                    },
                    ast::UnaryOp::UAdd => match val {
                        PyVal::Int(_) | PyVal::Float(_) => Ok(val),
                        _ => Err(SandboxError::Runtime("TypeError: operador unário '+' inválido".to_string())),
                    },
                    ast::UnaryOp::Not => Ok(PyVal::Bool(!val.is_truthy())),
                    _ => Err(SandboxError::Runtime("TypeError: operador unário não suportado".to_string())),
                }
            }
            ast::Expr::BoolOp(b) => match b.op {
                ast::BoolOp::And => {
                    let mut last = PyVal::Bool(true);
                    for val_expr in &b.values {
                        last = Self::eval_expr(vm, val_expr, locals)?;
                        if !last.is_truthy() {
                            return Ok(last);
                        }
                    }
                    Ok(last)
                }
                ast::BoolOp::Or => {
                    let mut last = PyVal::Bool(false);
                    for val_expr in &b.values {
                        last = Self::eval_expr(vm, val_expr, locals)?;
                        if last.is_truthy() {
                            return Ok(last);
                        }
                    }
                    Ok(last)
                }
            },
            ast::Expr::Compare(c) => {
                let mut left = Self::eval_expr(vm, &c.left, locals)?;
                for (op, comp_expr) in c.ops.iter().zip(c.comparators.iter()) {
                    let right = Self::eval_expr(vm, comp_expr, locals)?;
                    if !Self::apply_cmpop(op, &left, &right)? {
                        return Ok(PyVal::Bool(false));
                    }
                    left = right;
                }
                Ok(PyVal::Bool(true))
            }
            ast::Expr::IfExp(ie) => {
                let cond = Self::eval_expr(vm, &ie.test, locals)?;
                if cond.is_truthy() {
                    Self::eval_expr(vm, &ie.body, locals)
                } else {
                    Self::eval_expr(vm, &ie.orelse, locals)
                }
            }
            ast::Expr::List(l) => {
                let mut items = Vec::with_capacity(l.elts.len());
                for el in &l.elts {
                    items.push(Self::eval_expr(vm, el, locals)?);
                }
                Ok(PyVal::List(items))
            }
            ast::Expr::Tuple(t) => {
                let mut items = Vec::with_capacity(t.elts.len());
                for el in &t.elts {
                    items.push(Self::eval_expr(vm, el, locals)?);
                }
                Ok(PyVal::Tuple(items))
            }
            ast::Expr::Dict(d) => {
                let mut pairs = Vec::with_capacity(d.values.len());
                for (k_opt, v_expr) in d.keys.iter().zip(d.values.iter()) {
                    let k = match k_opt {
                        Some(ke) => Self::eval_expr(vm, ke, locals)?,
                        None => PyVal::None,
                    };
                    let v = Self::eval_expr(vm, v_expr, locals)?;
                    pairs.push((k, v));
                }
                Ok(PyVal::Dict(pairs))
            }
            ast::Expr::ListComp(lc) => {
                let mut out = Vec::new();
                Self::eval_list_comp(vm, lc, 0, locals, &mut out)?;
                Ok(PyVal::List(out))
            }
            ast::Expr::JoinedStr(js) => {
                let mut out = String::new();
                for part in &js.values {
                    match part {
                        ast::Expr::Constant(c) => {
                            out.push_str(&Self::eval_constant(&c.value)?.to_display_string());
                        }
                        ast::Expr::FormattedValue(fv) => {
                            let val = Self::eval_expr(vm, &fv.value, locals)?;
                            out.push_str(&val.to_display_string());
                        }
                        other => {
                            let val = Self::eval_expr(vm, other, locals)?;
                            out.push_str(&val.to_display_string());
                        }
                    }
                }
                vm.track_alloc(out.len())?;
                Ok(PyVal::Str(out))
            }
            ast::Expr::Subscript(s) => {
                let obj = Self::eval_expr(vm, &s.value, locals)?;
                if let ast::Expr::Slice(sl) = s.slice.as_ref() {
                    return Self::eval_slice(vm, &obj, sl, locals);
                }
                let idx_val = Self::eval_expr(vm, &s.slice, locals)?;
                match obj {
                    PyVal::List(items) | PyVal::Tuple(items) => {
                        let idx = Self::resolve_index(&idx_val, items.len())?;
                        Ok(items[idx].clone())
                    }
                    PyVal::Str(s_str) => {
                        let chars: Vec<char> = s_str.chars().collect();
                        let idx = Self::resolve_index(&idx_val, chars.len())?;
                        Ok(PyVal::Str(chars[idx].to_string()))
                    }
                    PyVal::Dict(pairs) => pairs
                        .into_iter()
                        .find(|(k, _)| k.py_eq(&idx_val))
                        .map(|(_, v)| v)
                        .ok_or_else(|| {
                            SandboxError::Runtime(format!(
                                "KeyError: {}",
                                idx_val.to_repr_string()
                            ))
                        }),
                    _ => Err(SandboxError::Runtime(
                        "TypeError: objeto não é indexável".to_string(),
                    )),
                }
            }
            ast::Expr::Call(call) => Self::eval_call(vm, call, locals),
            _ => Err(SandboxError::Runtime(
                "SyntaxError: expressão não suportada no sandbox".to_string(),
            )),
        }
    }

    fn eval_list_comp(
        vm: &mut VmState,
        lc: &ast::ExprListComp,
        gen_idx: usize,
        locals: &mut BTreeMap<String, PyVal>,
        out: &mut Vec<PyVal>,
    ) -> Result<(), SandboxError> {
        if gen_idx >= lc.generators.len() {
            let item = Self::eval_expr(vm, &lc.elt, locals)?;
            if out.len() >= MAX_COLLECTION_LEN {
                return Err(SandboxError::MemoryLimit(
                    "MemoryLimitError: compreensão de lista excedeu tamanho máximo".to_string(),
                ));
            }
            out.push(item);
            return Ok(());
        }
        let gen = &lc.generators[gen_idx];
        let iter_val = Self::eval_expr(vm, &gen.iter, locals)?;
        let items = Self::to_iterable(&iter_val)?;
        for item in items {
            vm.check_limits()?;
            Self::assign_target(vm, &gen.target, item, locals)?;
            let mut cond_ok = true;
            for cond in &gen.ifs {
                if !Self::eval_expr(vm, cond, locals)?.is_truthy() {
                    cond_ok = false;
                    break;
                }
            }
            if cond_ok {
                Self::eval_list_comp(vm, lc, gen_idx + 1, locals, out)?;
            }
        }
        Ok(())
    }

    fn eval_slice(
        vm: &mut VmState,
        obj: &PyVal,
        sl: &ast::ExprSlice,
        locals: &mut BTreeMap<String, PyVal>,
    ) -> Result<PyVal, SandboxError> {
        let lower = match &sl.lower {
            Some(e) => match Self::eval_expr(vm, e, locals)? {
                PyVal::Int(i) => Some(i),
                PyVal::None => None,
                _ => return Err(SandboxError::Runtime("TypeError: índice de slice deve ser inteiro".to_string())),
            },
            None => None,
        };
        let upper = match &sl.upper {
            Some(e) => match Self::eval_expr(vm, e, locals)? {
                PyVal::Int(i) => Some(i),
                PyVal::None => None,
                _ => return Err(SandboxError::Runtime("TypeError: índice de slice deve ser inteiro".to_string())),
            },
            None => None,
        };

        let clamp = |len: usize| -> (usize, usize) {
            let l = len as i64;
            let start = match lower {
                Some(i) => {
                    let norm = if i < 0 { l + i } else { i };
                    norm.clamp(0, l) as usize
                }
                None => 0,
            };
            let end = match upper {
                Some(i) => {
                    let norm = if i < 0 { l + i } else { i };
                    norm.clamp(0, l) as usize
                }
                None => len,
            };
            (start, end.max(start))
        };

        match obj {
            PyVal::List(items) => {
                let (s, e) = clamp(items.len());
                Ok(PyVal::List(items[s..e].to_vec()))
            }
            PyVal::Tuple(items) => {
                let (s, e) = clamp(items.len());
                Ok(PyVal::Tuple(items[s..e].to_vec()))
            }
            PyVal::Str(txt) => {
                let chars: Vec<char> = txt.chars().collect();
                let (s, e) = clamp(chars.len());
                Ok(PyVal::Str(chars[s..e].iter().collect()))
            }
            _ => Err(SandboxError::Runtime("TypeError: objeto não suporta fatiamento".to_string())),
        }
    }

    fn eval_constant(c: &ast::Constant) -> Result<PyVal, SandboxError> {
        match c {
            ast::Constant::None => Ok(PyVal::None),
            ast::Constant::Bool(b) => Ok(PyVal::Bool(*b)),
            ast::Constant::Int(i) => {
                let s = i.to_string();
                let parsed = s.parse::<i64>().map_err(|_| {
                    SandboxError::MemoryLimit(
                        "MemoryLimitError: inteiro excede capacidade segura".to_string(),
                    )
                })?;
                Ok(PyVal::Int(parsed))
            }
            ast::Constant::Float(f) => Ok(PyVal::Float(*f)),
            ast::Constant::Str(s) => Ok(PyVal::Str(s.clone())),
            _ => Err(SandboxError::Runtime(
                "TypeError: constante não suportada".to_string(),
            )),
        }
    }

    fn apply_binop(
        vm: &mut VmState,
        op: &ast::Operator,
        left: PyVal,
        right: PyVal,
    ) -> Result<PyVal, SandboxError> {
        match op {
            ast::Operator::Add => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) => Ok(PyVal::Int(a.saturating_add(b))),
                (PyVal::Float(a), PyVal::Float(b)) => Ok(PyVal::Float(a + b)),
                (PyVal::Int(a), PyVal::Float(b)) | (PyVal::Float(b), PyVal::Int(a)) => {
                    Ok(PyVal::Float(a as f64 + b))
                }
                (PyVal::Str(a), PyVal::Str(b)) => {
                    vm.track_alloc(a.len() + b.len())?;
                    Ok(PyVal::Str(format!("{}{}", a, b)))
                }
                (PyVal::List(mut a), PyVal::List(b)) => {
                    if a.len() + b.len() > MAX_COLLECTION_LEN {
                        return Err(SandboxError::MemoryLimit(
                            "MemoryLimitError: lista excedeu capacidade".to_string(),
                        ));
                    }
                    a.extend(b);
                    Ok(PyVal::List(a))
                }
                _ => Err(SandboxError::Runtime("TypeError: tipos incompatíveis para '+'".to_string())),
            },
            ast::Operator::Sub => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) => Ok(PyVal::Int(a.saturating_sub(b))),
                (PyVal::Float(a), PyVal::Float(b)) => Ok(PyVal::Float(a - b)),
                (PyVal::Int(a), PyVal::Float(b)) => Ok(PyVal::Float(a as f64 - b)),
                (PyVal::Float(a), PyVal::Int(b)) => Ok(PyVal::Float(a - b as f64)),
                _ => Err(SandboxError::Runtime("TypeError: tipos incompatíveis para '-'".to_string())),
            },
            ast::Operator::Mult => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) => Ok(PyVal::Int(a.saturating_mul(b))),
                (PyVal::Float(a), PyVal::Float(b)) => Ok(PyVal::Float(a * b)),
                (PyVal::Int(a), PyVal::Float(b)) | (PyVal::Float(b), PyVal::Int(a)) => {
                    Ok(PyVal::Float(a as f64 * b))
                }
                (PyVal::Str(s), PyVal::Int(n)) | (PyVal::Int(n), PyVal::Str(s)) => {
                    if n <= 0 {
                        return Ok(PyVal::Str(String::new()));
                    }
                    let count = n as usize;
                    let total_bytes = s.len().saturating_mul(count);
                    if total_bytes > MAX_MEMORY_BYTES / 4 {
                        return Err(SandboxError::MemoryLimit(
                            "MemoryLimitError: multiplicação de string excedeu limite seguro de memória".to_string(),
                        ));
                    }
                    vm.track_alloc(total_bytes)?;
                    Ok(PyVal::Str(s.repeat(count)))
                }
                (PyVal::List(items), PyVal::Int(n)) | (PyVal::Int(n), PyVal::List(items)) => {
                    if n <= 0 {
                        return Ok(PyVal::List(vec![]));
                    }
                    let count = n as usize;
                    if items.len().saturating_mul(count) > MAX_COLLECTION_LEN {
                        return Err(SandboxError::MemoryLimit(
                            "MemoryLimitError: multiplicação de lista excedeu limite seguro".to_string(),
                        ));
                    }
                    let mut out = Vec::with_capacity(items.len() * count);
                    for _ in 0..count {
                        out.extend(items.clone());
                    }
                    Ok(PyVal::List(out))
                }
                _ => Err(SandboxError::Runtime("TypeError: tipos incompatíveis para '*'".to_string())),
            },
            ast::Operator::Div => {
                let a = left.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: divisão inválida".to_string()))?;
                let b = right.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: divisão inválida".to_string()))?;
                if b == 0.0 {
                    return Err(SandboxError::Runtime("ZeroDivisionError: division by zero".to_string()));
                }
                Ok(PyVal::Float(a / b))
            }
            ast::Operator::FloorDiv => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) => {
                    if b == 0 {
                        return Err(SandboxError::Runtime("ZeroDivisionError: integer division by zero".to_string()));
                    }
                    Ok(PyVal::Int(a.div_euclid(b)))
                }
                (a, b) => {
                    let fa = a.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '//' inválido".to_string()))?;
                    let fb = b.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '//' inválido".to_string()))?;
                    if fb == 0.0 {
                        return Err(SandboxError::Runtime("ZeroDivisionError: division by zero".to_string()));
                    }
                    Ok(PyVal::Float((fa / fb).floor()))
                }
            },
            ast::Operator::Mod => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) => {
                    if b == 0 {
                        return Err(SandboxError::Runtime("ZeroDivisionError: modulo by zero".to_string()));
                    }
                    Ok(PyVal::Int(a.rem_euclid(b)))
                }
                (a, b) => {
                    let fa = a.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '%' inválido".to_string()))?;
                    let fb = b.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '%' inválido".to_string()))?;
                    if fb == 0.0 {
                        return Err(SandboxError::Runtime("ZeroDivisionError: modulo by zero".to_string()));
                    }
                    Ok(PyVal::Float(((fa % fb) + fb) % fb))
                }
            },
            ast::Operator::Pow => match (left, right) {
                (PyVal::Int(a), PyVal::Int(b)) if b >= 0 => {
                    if b > 62 && a.abs() > 1 {
                        return Err(SandboxError::MemoryLimit(
                            "MemoryLimitError: potência exponencial excede limite seguro".to_string(),
                        ));
                    }
                    Ok(PyVal::Int(a.saturating_pow(b as u32)))
                }
                (a, b) => {
                    let fa = a.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '**' inválido".to_string()))?;
                    let fb = b.as_f64().ok_or_else(|| SandboxError::Runtime("TypeError: '**' inválido".to_string()))?;
                    Ok(PyVal::Float(fa.powf(fb)))
                }
            },
            _ => Err(SandboxError::Runtime("TypeError: operador binário não suportado".to_string())),
        }
    }

    fn apply_cmpop(op: &ast::CmpOp, left: &PyVal, right: &PyVal) -> Result<bool, SandboxError> {
        match op {
            ast::CmpOp::Eq => Ok(left.py_eq(right)),
            ast::CmpOp::NotEq => Ok(!left.py_eq(right)),
            ast::CmpOp::Lt | ast::CmpOp::LtE | ast::CmpOp::Gt | ast::CmpOp::GtE => {
                let ord = match (left, right) {
                    (PyVal::Str(a), PyVal::Str(b)) => Some(a.cmp(b)),
                    _ => match (left.as_f64(), right.as_f64()) {
                        (Some(a), Some(b)) => a.partial_cmp(&b),
                        _ => None,
                    },
                };
                let ord = ord.ok_or_else(|| {
                    SandboxError::Runtime("TypeError: comparação entre tipos incompatíveis".to_string())
                })?;
                Ok(match op {
                    ast::CmpOp::Lt => ord == std::cmp::Ordering::Less,
                    ast::CmpOp::LtE => ord != std::cmp::Ordering::Greater,
                    ast::CmpOp::Gt => ord == std::cmp::Ordering::Greater,
                    ast::CmpOp::GtE => ord != std::cmp::Ordering::Less,
                    _ => unreachable!(),
                })
            }
            ast::CmpOp::In | ast::CmpOp::NotIn => {
                let found = match right {
                    PyVal::List(items) | PyVal::Tuple(items) => items.iter().any(|x| x.py_eq(left)),
                    PyVal::Str(haystack) => match left {
                        PyVal::Str(needle) => haystack.contains(needle.as_str()),
                        _ => return Err(SandboxError::Runtime("TypeError: 'in <string>' requer string".to_string())),
                    },
                    PyVal::Dict(pairs) => pairs.iter().any(|(k, _)| k.py_eq(left)),
                    _ => return Err(SandboxError::Runtime("TypeError: operando direito de 'in' não é iterável".to_string())),
                };
                Ok(if matches!(op, ast::CmpOp::In) { found } else { !found })
            }
            ast::CmpOp::Is => Ok(matches!((left, right), (PyVal::None, PyVal::None)) || left.py_eq(right)),
            ast::CmpOp::IsNot => Ok(!(matches!((left, right), (PyVal::None, PyVal::None)) || left.py_eq(right))),
        }
    }

    fn eval_call(
        vm: &mut VmState,
        call: &ast::ExprCall,
        locals: &mut BTreeMap<String, PyVal>,
    ) -> Result<PyVal, SandboxError> {
        let mut args = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            args.push(Self::eval_expr(vm, arg, locals)?);
        }

        // Chamada de método: obj.method(...)
        if let ast::Expr::Attribute(attr) = call.func.as_ref() {
            let method = attr.attr.as_str();
            if let ast::Expr::Name(var_name) = attr.value.as_ref() {
                let name = var_name.id.to_string();
                let current_obj = locals
                    .get(&name)
                    .or_else(|| vm.globals.get(&name))
                    .cloned()
                    .ok_or_else(|| SandboxError::Runtime(format!("NameError: '{}'", name)))?;

                if let PyVal::List(mut list) = current_obj {
                    match method {
                        "append" => {
                            let val = args.into_iter().next().ok_or_else(|| {
                                SandboxError::Runtime("TypeError: append() espera 1 argumento".to_string())
                            })?;
                            if list.len() >= MAX_COLLECTION_LEN {
                                return Err(SandboxError::MemoryLimit("MemoryLimitError: lista excedeu limite".to_string()));
                            }
                            vm.track_alloc(val.approx_bytes())?;
                            list.push(val);
                            locals.insert(name, PyVal::List(list));
                            return Ok(PyVal::None);
                        }
                        "pop" => {
                            let popped = if let Some(idx_val) = args.first() {
                                let idx = Self::resolve_index(idx_val, list.len())?;
                                list.remove(idx)
                            } else {
                                list.pop().ok_or_else(|| {
                                    SandboxError::Runtime("IndexError: pop from empty list".to_string())
                                })?
                            };
                            locals.insert(name, PyVal::List(list));
                            return Ok(popped);
                        }
                        "extend" => {
                            let other = args.into_iter().next().ok_or_else(|| {
                                SandboxError::Runtime("TypeError: extend() espera 1 argumento".to_string())
                            })?;
                            let items = Self::to_iterable(&other)?;
                            if list.len() + items.len() > MAX_COLLECTION_LEN {
                                return Err(SandboxError::MemoryLimit("MemoryLimitError: lista excedeu limite".to_string()));
                            }
                            list.extend(items);
                            locals.insert(name, PyVal::List(list));
                            return Ok(PyVal::None);
                        }
                        "clear" => {
                            list.clear();
                            locals.insert(name, PyVal::List(list));
                            return Ok(PyVal::None);
                        }
                        "reverse" => {
                            list.reverse();
                            locals.insert(name, PyVal::List(list));
                            return Ok(PyVal::None);
                        }
                        "sort" => {
                            list.sort_by(|a, b| {
                                a.as_f64()
                                    .partial_cmp(&b.as_f64())
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                            locals.insert(name, PyVal::List(list));
                            return Ok(PyVal::None);
                        }
                        _ => {}
                    }
                }
            }

            let target_val = Self::eval_expr(vm, &attr.value, locals)?;
            return Self::eval_method_on_value(target_val, method, args);
        }

        if let ast::Expr::Name(n) = call.func.as_ref() {
            let fname = n.id.as_str();
            // Builtins
            match fname {
                "print" => {
                    let rendered: Vec<String> =
                        args.iter().map(|a| a.to_display_string()).collect();
                    vm.push_stdout(&format!("{}\n", rendered.join(" ")));
                    return Ok(PyVal::None);
                }
                "len" => {
                    let arg = args.first().ok_or_else(|| SandboxError::Runtime("TypeError: len() espera 1 argumento".to_string()))?;
                    return match arg {
                        PyVal::List(l) | PyVal::Tuple(l) => Ok(PyVal::Int(l.len() as i64)),
                        PyVal::Str(s) => Ok(PyVal::Int(s.chars().count() as i64)),
                        PyVal::Dict(d) => Ok(PyVal::Int(d.len() as i64)),
                        _ => Err(SandboxError::Runtime("TypeError: objeto não possui len()".to_string())),
                    };
                }
                "range" => {
                    let (start, stop, step) = match args.len() {
                        1 => (0, Self::expect_int(&args[0])?, 1),
                        2 => (Self::expect_int(&args[0])?, Self::expect_int(&args[1])?, 1),
                        3 => (
                            Self::expect_int(&args[0])?,
                            Self::expect_int(&args[1])?,
                            Self::expect_int(&args[2])?,
                        ),
                        _ => return Err(SandboxError::Runtime("TypeError: range() espera 1 a 3 argumentos".to_string())),
                    };
                    if step == 0 {
                        return Err(SandboxError::Runtime("ValueError: range() arg 3 must not be zero".to_string()));
                    }
                    let mut out = Vec::new();
                    let mut cur = start;
                    while (step > 0 && cur < stop) || (step < 0 && cur > stop) {
                        if out.len() >= MAX_COLLECTION_LEN {
                            return Err(SandboxError::MemoryLimit(
                                "MemoryLimitError: range() excedeu limite seguro".to_string(),
                            ));
                        }
                        out.push(PyVal::Int(cur));
                        cur += step;
                    }
                    return Ok(PyVal::List(out));
                }
                "round" => {
                    let num = args
                        .first()
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| SandboxError::Runtime("TypeError: round() espera número".to_string()))?;
                    let digits = args.get(1).and_then(|v| match v {
                        PyVal::Int(i) => Some(*i),
                        _ => None,
                    });
                    return match digits {
                        Some(d) => {
                            let factor = 10f64.powi(d as i32);
                            Ok(PyVal::Float((num * factor).round() / factor))
                        }
                        None => Ok(PyVal::Int(num.round() as i64)),
                    };
                }
                "abs" => {
                    let arg = args.first().ok_or_else(|| SandboxError::Runtime("TypeError: abs() espera 1 argumento".to_string()))?;
                    return match arg {
                        PyVal::Int(i) => Ok(PyVal::Int(i.abs())),
                        PyVal::Float(f) => Ok(PyVal::Float(f.abs())),
                        _ => Err(SandboxError::Runtime("TypeError: abs() espera número".to_string())),
                    };
                }
                "min" | "max" | "sum" => {
                    let items = if args.len() == 1 {
                        Self::to_iterable(&args[0])?
                    } else {
                        args.clone()
                    };
                    if fname == "sum" {
                        let mut acc = 0.0;
                        let mut is_float = false;
                        for it in &items {
                            match it {
                                PyVal::Int(i) => acc += *i as f64,
                                PyVal::Float(f) => {
                                    is_float = true;
                                    acc += *f;
                                }
                                _ => return Err(SandboxError::Runtime("TypeError: sum() requer números".to_string())),
                            }
                        }
                        return Ok(if is_float {
                            PyVal::Float(acc)
                        } else {
                            PyVal::Int(acc as i64)
                        });
                    }
                    let mut best = items.first().cloned().ok_or_else(|| {
                        SandboxError::Runtime("ValueError: sequência vazia".to_string())
                    })?;
                    for it in items.into_iter().skip(1) {
                        let cmp = if fname == "min" {
                            Self::apply_cmpop(&ast::CmpOp::Lt, &it, &best)?
                        } else {
                            Self::apply_cmpop(&ast::CmpOp::Gt, &it, &best)?
                        };
                        if cmp {
                            best = it;
                        }
                    }
                    return Ok(best);
                }
                "int" => {
                    let arg = args.first().ok_or_else(|| SandboxError::Runtime("TypeError: int() espera argumento".to_string()))?;
                    return match arg {
                        PyVal::Int(i) => Ok(PyVal::Int(*i)),
                        PyVal::Float(f) => Ok(PyVal::Int(*f as i64)),
                        PyVal::Bool(b) => Ok(PyVal::Int(if *b { 1 } else { 0 })),
                        PyVal::Str(s) => s.trim().parse::<i64>().map(PyVal::Int).map_err(|_| {
                            SandboxError::Runtime("ValueError: invalid literal for int()".to_string())
                        }),
                        _ => Err(SandboxError::Runtime("TypeError: conversão inválida para int()".to_string())),
                    };
                }
                "float" => {
                    let arg = args.first().ok_or_else(|| SandboxError::Runtime("TypeError: float() espera argumento".to_string()))?;
                    return match arg {
                        PyVal::Int(i) => Ok(PyVal::Float(*i as f64)),
                        PyVal::Float(f) => Ok(PyVal::Float(*f)),
                        PyVal::Str(s) => s.trim().parse::<f64>().map(PyVal::Float).map_err(|_| {
                            SandboxError::Runtime("ValueError: could not convert string to float".to_string())
                        }),
                        _ => Err(SandboxError::Runtime("TypeError: conversão inválida para float()".to_string())),
                    };
                }
                "str" => {
                    let arg = args.first().cloned().unwrap_or(PyVal::Str(String::new()));
                    return Ok(PyVal::Str(arg.to_display_string()));
                }
                "bool" => {
                    let arg = args.first().cloned().unwrap_or(PyVal::Bool(false));
                    return Ok(PyVal::Bool(arg.is_truthy()));
                }
                "list" => {
                    if args.is_empty() {
                        return Ok(PyVal::List(vec![]));
                    }
                    return Ok(PyVal::List(Self::to_iterable(&args[0])?));
                }
                "enumerate" => {
                    let items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: enumerate() espera iterável".to_string())
                    })?)?;
                    return Ok(PyVal::List(
                        items
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| PyVal::Tuple(vec![PyVal::Int(i as i64), v]))
                            .collect(),
                    ));
                }
                "sorted" => {
                    let mut items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: sorted() espera iterável".to_string())
                    })?)?;
                    items.sort_by(|a, b| {
                        a.as_f64()
                            .partial_cmp(&b.as_f64())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    return Ok(PyVal::List(items));
                }
                "reversed" => {
                    let mut items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: reversed() espera iterável".to_string())
                    })?)?;
                    items.reverse();
                    return Ok(PyVal::List(items));
                }
                "any" => {
                    let items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: any() espera iterável".to_string())
                    })?)?;
                    return Ok(PyVal::Bool(items.iter().any(|x| x.is_truthy())));
                }
                "all" => {
                    let items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: all() espera iterável".to_string())
                    })?)?;
                    return Ok(PyVal::Bool(items.iter().all(|x| x.is_truthy())));
                }
                _ => {}
            }

            if vm.functions.contains_key(fname) {
                return Self::call_user_fn(vm, fname, args);
            }

            return Err(SandboxError::Runtime(format!(
                "NameError: name '{}' is not defined",
                fname
            )));
        }

        Err(SandboxError::Runtime(
            "TypeError: objeto não é chamável".to_string(),
        ))
    }

    fn eval_method_on_value(
        target: PyVal,
        method: &str,
        args: Vec<PyVal>,
    ) -> Result<PyVal, SandboxError> {
        match target {
            PyVal::Str(s) => match method {
                "upper" => Ok(PyVal::Str(s.to_uppercase())),
                "lower" => Ok(PyVal::Str(s.to_lowercase())),
                "strip" => Ok(PyVal::Str(s.trim().to_string())),
                "split" => {
                    let parts: Vec<PyVal> = if let Some(PyVal::Str(sep)) = args.first() {
                        s.split(sep.as_str())
                            .map(|p| PyVal::Str(p.to_string()))
                            .collect()
                    } else {
                        s.split_whitespace()
                            .map(|p| PyVal::Str(p.to_string()))
                            .collect()
                    };
                    Ok(PyVal::List(parts))
                }
                "join" => {
                    let items = Self::to_iterable(args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: join() espera iterável".to_string())
                    })?)?;
                    let strs: Vec<String> = items.into_iter().map(|i| i.to_display_string()).collect();
                    Ok(PyVal::Str(strs.join(&s)))
                }
                _ => Err(SandboxError::Runtime(format!(
                    "AttributeError: 'str' object has no attribute '{}'",
                    method
                ))),
            },
            PyVal::Dict(pairs) => match method {
                "get" => {
                    let key = args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: get() espera chave".to_string())
                    })?;
                    let default = args.get(1).cloned().unwrap_or(PyVal::None);
                    Ok(pairs
                        .into_iter()
                        .find(|(k, _)| k.py_eq(key))
                        .map(|(_, v)| v)
                        .unwrap_or(default))
                }
                "keys" => Ok(PyVal::List(pairs.into_iter().map(|(k, _)| k).collect())),
                "values" => Ok(PyVal::List(pairs.into_iter().map(|(_, v)| v).collect())),
                "items" => Ok(PyVal::List(
                    pairs
                        .into_iter()
                        .map(|(k, v)| PyVal::Tuple(vec![k, v]))
                        .collect(),
                )),
                _ => Err(SandboxError::Runtime(format!(
                    "AttributeError: 'dict' object has no attribute '{}'",
                    method
                ))),
            },
            PyVal::List(items) => match method {
                "count" => {
                    let needle = args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: count() espera argumento".to_string())
                    })?;
                    let cnt = items.iter().filter(|x| x.py_eq(needle)).count();
                    Ok(PyVal::Int(cnt as i64))
                }
                "index" => {
                    let needle = args.first().ok_or_else(|| {
                        SandboxError::Runtime("TypeError: index() espera argumento".to_string())
                    })?;
                    items
                        .iter()
                        .position(|x| x.py_eq(needle))
                        .map(|i| PyVal::Int(i as i64))
                        .ok_or_else(|| SandboxError::Runtime("ValueError: valor não encontrado na lista".to_string()))
                }
                _ => Err(SandboxError::Runtime(format!(
                    "AttributeError: 'list' object has no attribute '{}'",
                    method
                ))),
            },
            _ => Err(SandboxError::Runtime(format!(
                "AttributeError: método '{}' não suportado",
                method
            ))),
        }
    }

    fn expect_int(v: &PyVal) -> Result<i64, SandboxError> {
        match v {
            PyVal::Int(i) => Ok(*i),
            PyVal::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(SandboxError::Runtime(
                "TypeError: argumento inteiro esperado".to_string(),
            )),
        }
    }

    fn call_user_fn(
        vm: &mut VmState,
        fn_name: &str,
        args: Vec<PyVal>,
    ) -> Result<PyVal, SandboxError> {
        vm.check_limits()?;
        if vm.recursion_depth >= MAX_RECURSION_DEPTH {
            return Err(SandboxError::Runtime(
                "RecursionError: maximum recursion depth exceeded".to_string(),
            ));
        }

        let fn_def = vm
            .functions
            .get(fn_name)
            .cloned()
            .ok_or_else(|| SandboxError::Runtime(format!("NameError: '{}'", fn_name)))?;

        let mut fn_locals = BTreeMap::new();
        for (idx, param_name) in fn_def.params.iter().enumerate() {
            if let Some(arg_val) = args.get(idx) {
                fn_locals.insert(param_name.clone(), arg_val.clone());
            } else if let Some(Some(default_expr)) = fn_def.defaults.get(idx) {
                let default_val = Self::eval_expr(vm, default_expr, &mut fn_locals)?;
                fn_locals.insert(param_name.clone(), default_val);
            } else {
                return Err(SandboxError::Runtime(format!(
                    "TypeError: {}() missing required positional argument: '{}'",
                    fn_name, param_name
                )));
            }
        }

        vm.recursion_depth += 1;
        let flow = Self::exec_block(vm, &fn_def.body, &mut fn_locals, true);
        vm.recursion_depth -= 1;

        match flow? {
            ExecFlow::Return(val) => Ok(val),
            _ => Ok(PyVal::None),
        }
    }
}
