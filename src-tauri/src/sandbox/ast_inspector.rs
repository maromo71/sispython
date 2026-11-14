use crate::agents::{AstInspectionReport, LevelRuleConfig, TestCaseSpec};
use rustpython_parser::{ast, Parse};
use std::collections::HashSet;

pub struct AstInspector;

impl AstInspector {
    /// Analisa a Árvore Sintática Abstrata (AST) do código Python do aluno em < 5ms.
    pub fn inspect(
        source_code: &str,
        rules: &LevelRuleConfig,
        test_cases: &[TestCaseSpec],
    ) -> AstInspectionReport {
        let parsed = match ast::Suite::parse(source_code, "<kiosk_submission>") {
            Ok(suite) => suite,
            Err(parse_err) => {
                let offset = usize::from(parse_err.offset);
                let line = source_code[..offset.min(source_code.len())]
                    .bytes()
                    .filter(|&b| b == b'\n')
                    .count()
                    + 1;
                return AstInspectionReport {
                    is_valid: false,
                    detected_nodes: vec![],
                    missing_required_nodes: vec![],
                    triggered_forbidden_nodes: vec![],
                    suspected_hardcode: false,
                    syntax_error_detail: Some((parse_err.error.to_string(), line)),
                };
            }
        };

        let mut detected = HashSet::new();
        let mut return_literals = Vec::new();
        let mut print_literals = Vec::new();

        for stmt in &parsed {
            Self::walk_stmt(stmt, &mut detected, &mut return_literals, &mut print_literals);
        }

        let mut detected_vec: Vec<String> = detected.iter().cloned().collect();
        detected_vec.sort();

        let missing_required: Vec<String> = rules
            .must_contain_nodes
            .iter()
            .filter(|req| !detected.contains(*req))
            .cloned()
            .collect();

        let triggered_forbidden: Vec<String> = rules
            .forbidden_nodes
            .iter()
            .filter(|forb| detected.contains(*forb))
            .cloned()
            .collect();

        let suspected_hardcode = Self::detect_hardcoded_solution(
            &detected,
            &return_literals,
            &print_literals,
            test_cases,
        );

        let is_valid =
            missing_required.is_empty() && triggered_forbidden.is_empty() && !suspected_hardcode;

        AstInspectionReport {
            is_valid,
            detected_nodes: detected_vec,
            missing_required_nodes: missing_required,
            triggered_forbidden_nodes: triggered_forbidden,
            suspected_hardcode,
            syntax_error_detail: None,
        }
    }

    fn walk_stmt(
        stmt: &ast::Stmt,
        detected: &mut HashSet<String>,
        return_literals: &mut Vec<String>,
        print_literals: &mut Vec<String>,
    ) {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                detected.insert("FunctionDef".to_string());
                for s in &f.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                detected.insert("FunctionDef".to_string());
                for s in &f.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::For(f) => {
                detected.insert("For".to_string());
                Self::walk_expr(&f.iter, detected, print_literals);
                for s in &f.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
                for s in &f.orelse {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::While(w) => {
                detected.insert("While".to_string());
                Self::walk_expr(&w.test, detected, print_literals);
                for s in &w.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
                for s in &w.orelse {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::If(i) => {
                detected.insert("If".to_string());
                Self::walk_expr(&i.test, detected, print_literals);
                for s in &i.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
                for s in &i.orelse {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::Assign(a) => {
                detected.insert("Assign".to_string());
                Self::walk_expr(&a.value, detected, print_literals);
            }
            ast::Stmt::AugAssign(aa) => {
                detected.insert("Assign".to_string());
                detected.insert("BinOp".to_string());
                Self::walk_expr(&aa.value, detected, print_literals);
            }
            ast::Stmt::AnnAssign(aa) => {
                detected.insert("Assign".to_string());
                if let Some(val) = &aa.value {
                    Self::walk_expr(val, detected, print_literals);
                }
            }
            ast::Stmt::Return(r) => {
                detected.insert("Return".to_string());
                if let Some(val) = &r.value {
                    if let ast::Expr::Constant(c) = val.as_ref() {
                        return_literals.push(Self::constant_to_string(&c.value));
                    }
                    Self::walk_expr(val, detected, print_literals);
                }
            }
            ast::Stmt::Break(_) => {
                detected.insert("Break".to_string());
            }
            ast::Stmt::Continue(_) => {
                detected.insert("Continue".to_string());
            }
            ast::Stmt::Import(_) | ast::Stmt::ImportFrom(_) => {
                detected.insert("Import".to_string());
            }
            ast::Stmt::Expr(e) => {
                Self::walk_expr(&e.value, detected, print_literals);
            }
            ast::Stmt::Try(t) => {
                for s in &t.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
                for s in &t.orelse {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
                for s in &t.finalbody {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            ast::Stmt::With(w) => {
                for s in &w.body {
                    Self::walk_stmt(s, detected, return_literals, print_literals);
                }
            }
            _ => {}
        }
    }

    fn walk_expr(
        expr: &ast::Expr,
        detected: &mut HashSet<String>,
        print_literals: &mut Vec<String>,
    ) {
        match expr {
            ast::Expr::BinOp(b) => {
                detected.insert("BinOp".to_string());
                Self::walk_expr(&b.left, detected, print_literals);
                Self::walk_expr(&b.right, detected, print_literals);
            }
            ast::Expr::UnaryOp(u) => {
                Self::walk_expr(&u.operand, detected, print_literals);
            }
            ast::Expr::BoolOp(b) => {
                detected.insert("Compare".to_string());
                for val in &b.values {
                    Self::walk_expr(val, detected, print_literals);
                }
            }
            ast::Expr::Compare(c) => {
                detected.insert("Compare".to_string());
                Self::walk_expr(&c.left, detected, print_literals);
                for comp in &c.comparators {
                    Self::walk_expr(comp, detected, print_literals);
                }
            }
            ast::Expr::IfExp(ie) => {
                detected.insert("If".to_string());
                Self::walk_expr(&ie.test, detected, print_literals);
                Self::walk_expr(&ie.body, detected, print_literals);
                Self::walk_expr(&ie.orelse, detected, print_literals);
            }
            ast::Expr::Call(call) => {
                if let ast::Expr::Name(n) = call.func.as_ref() {
                    let fname = n.id.as_str();
                    if fname == "eval" {
                        detected.insert("Eval".to_string());
                    } else if fname == "exec" {
                        detected.insert("Exec".to_string());
                    } else if fname == "__import__" {
                        detected.insert("Import".to_string());
                    } else if fname == "print" {
                        for arg in &call.args {
                            if let ast::Expr::Constant(c) = arg {
                                print_literals.push(Self::constant_to_string(&c.value));
                            }
                        }
                    }
                }
                Self::walk_expr(&call.func, detected, print_literals);
                for arg in &call.args {
                    Self::walk_expr(arg, detected, print_literals);
                }
                for kw in &call.keywords {
                    Self::walk_expr(&kw.value, detected, print_literals);
                }
            }
            ast::Expr::Attribute(a) => {
                Self::walk_expr(&a.value, detected, print_literals);
            }
            ast::Expr::Subscript(s) => {
                Self::walk_expr(&s.value, detected, print_literals);
                Self::walk_expr(&s.slice, detected, print_literals);
            }
            ast::Expr::List(l) => {
                for el in &l.elts {
                    Self::walk_expr(el, detected, print_literals);
                }
            }
            ast::Expr::Tuple(t) => {
                for el in &t.elts {
                    Self::walk_expr(el, detected, print_literals);
                }
            }
            ast::Expr::Dict(d) => {
                for k in d.keys.iter().flatten() {
                    Self::walk_expr(k, detected, print_literals);
                }
                for v in &d.values {
                    Self::walk_expr(v, detected, print_literals);
                }
            }
            ast::Expr::ListComp(lc) => {
                for gen in &lc.generators {
                    Self::walk_expr(&gen.iter, detected, print_literals);
                    for cond in &gen.ifs {
                        Self::walk_expr(cond, detected, print_literals);
                    }
                }
                Self::walk_expr(&lc.elt, detected, print_literals);
            }
            _ => {}
        }
    }

    fn constant_to_string(c: &ast::Constant) -> String {
        match c {
            ast::Constant::Int(i) => i.to_string(),
            ast::Constant::Float(f) => f.to_string(),
            ast::Constant::Str(s) => s.clone(),
            ast::Constant::Bool(b) => b.to_string(),
            ast::Constant::None => "None".to_string(),
            _ => format!("{:?}", c),
        }
    }

    fn detect_hardcoded_solution(
        detected: &HashSet<String>,
        return_literals: &[String],
        print_literals: &[String],
        test_cases: &[TestCaseSpec],
    ) -> bool {
        let has_logic = detected.contains("BinOp")
            || detected.contains("Compare")
            || detected.contains("For")
            || detected.contains("While");

        if !has_logic && !return_literals.is_empty() && !test_cases.is_empty() {
            return true;
        }

        // Também detecta `print(<literal_esperado>)` sem lógica estrutural
        if !has_logic && !print_literals.is_empty() && !test_cases.is_empty() {
            for lit in print_literals {
                for tc in test_cases {
                    let clean_expected = tc.expected_output_json.trim_matches('"');
                    if lit == clean_expected || lit == tc.expected_output_json.trim() {
                        return true;
                    }
                }
            }
        }

        false
    }
}
