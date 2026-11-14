use crate::agents::offline_rules::OfflineSocraticEngine;
use crate::agents::{AstInspectionReport, LevelRuleConfig, SocraticHintResponse};
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct SocraticTutorAgent {
    http_client: Client,
    api_key: Option<String>,
    base_url: String,
    ollama_endpoint: Option<String>,
    ollama_model: String,
}

impl SocraticTutorAgent {
    pub fn new(api_key: Option<String>) -> Self {
        let base_url = std::env::var("GEMINI_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_GEMINI_BASE_URL.to_string());
        let timeout_ms = std::env::var("GEMINI_REQUEST_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(8000);
        let ollama_enabled = std::env::var("LOCAL_LLM_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let ollama_endpoint = if ollama_enabled {
            Some(
                std::env::var("LOCAL_LLM_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:11434/api/generate".to_string()),
            )
        } else {
            None
        };
        let ollama_model =
            std::env::var("LOCAL_LLM_MODEL").unwrap_or_else(|_| "qwen2.5-coder:3b".to_string());

        let http_client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .unwrap_or_default();

        Self {
            http_client,
            api_key,
            base_url,
            ollama_endpoint,
            ollama_model,
        }
    }

    pub fn with_base_url(api_key: Option<String>, base_url: impl Into<String>) -> Self {
        let mut agent = Self::new(api_key);
        let url = base_url.into();
        if url.starts_with("http://127.0.0.1") || url.starts_with("http://localhost") {
            agent.http_client = Client::builder()
                .no_proxy()
                .timeout(Duration::from_millis(8000))
                .build()
                .unwrap_or_default();
        }
        agent.base_url = url;
        agent
    }

    /// Ponto de entrada principal: tenta os modelos gratuitos do Google AI Studio em cascata
    /// e aciona o motor determinístico local em Rust caso esteja offline ou sem cota.
    pub async fn generate_socratic_guidance(
        &self,
        student_code: &str,
        rules: &LevelRuleConfig,
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
    ) -> SocraticHintResponse {
        self.generate_socratic_guidance_with_error(
            student_code,
            rules,
            ast_report,
            failed_concept_tag,
            is_timeout_loop,
            None,
        )
        .await
    }

    pub async fn generate_socratic_guidance_with_error(
        &self,
        student_code: &str,
        rules: &LevelRuleConfig,
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
        runtime_error: Option<&str>,
    ) -> SocraticHintResponse {
        // Regra de Negócio: Em fases BOSS, dicas socráticas proativas são bloqueadas
        if rules.level_type == "BOSS" {
            return SocraticHintResponse {
                diagnostic_category: "BOSS_RESTRICTION".to_string(),
                friendly_explanation: "Você está em um Desafio Chefão! Aqui o seu conhecimento é colocado à prova sem auxílio externo.".to_string(),
                socratic_question: "Revise os conceitos das 4 fases anteriores deste módulo: quais estruturas você já dominou que se encaixam neste problema?".to_string(),
                focus_line: 1,
                conceptual_reminder: "Desafios Chefão exigem autonomia total para liberar a insígnia do módulo.".to_string(),
                provider_used: "LOCAL_RUST_ENGINE".to_string(),
            };
        }

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
                    if let Ok(mut response) = self
                        .call_gemini_free_tier(
                            key,
                            model_name,
                            student_code,
                            rules,
                            ast_report,
                            failed_concept_tag,
                            is_timeout_loop,
                            runtime_error,
                        )
                        .await
                    {
                        response.friendly_explanation =
                            Self::sanitize_no_code_leak(&response.friendly_explanation);
                        response.socratic_question =
                            Self::sanitize_no_code_leak(&response.socratic_question);
                        response.conceptual_reminder =
                            Self::sanitize_no_code_leak(&response.conceptual_reminder);
                        response.provider_used = provider_tag.to_string();
                        return response;
                    }
                }
            }
        }

        if let Some(endpoint) = &self.ollama_endpoint {
            if let Ok(mut response) = self
                .call_ollama_local(
                    endpoint,
                    student_code,
                    rules,
                    ast_report,
                    failed_concept_tag,
                    is_timeout_loop,
                )
                .await
            {
                response.friendly_explanation =
                    Self::sanitize_no_code_leak(&response.friendly_explanation);
                response.socratic_question =
                    Self::sanitize_no_code_leak(&response.socratic_question);
                response.conceptual_reminder =
                    Self::sanitize_no_code_leak(&response.conceptual_reminder);
                response.provider_used = "LOCAL_OLLAMA".to_string();
                return response;
            }
        }

        // Fallback 100% Offline Determinístico em Rust
        OfflineSocraticEngine::diagnose(
            ast_report,
            failed_concept_tag,
            is_timeout_loop,
            runtime_error,
        )
    }

    #[allow(clippy::too_many_arguments)]
    async fn call_gemini_free_tier(
        &self,
        api_key: &str,
        model: &str,
        student_code: &str,
        rules: &LevelRuleConfig,
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
        runtime_error: Option<&str>,
    ) -> Result<SocraticHintResponse, String> {
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/{}:generateContent?key={}", base, model, api_key);

        let system_prompt = "Você é um Mentor Socrático de Lógica de Programação em Português do Brasil. \
            REGRA INVIOLÁVEL: NUNCA escreva o código de solução, nunca forneça blocos de código prontos. \
            Seu papel é explicar o comportamento atual do código do aluno de forma acolhedora e fazer UMA pergunta instigadora \
            que leve o próprio aluno a descobrir como corrigir a linha onde o raciocínio desviou.";

        let user_context = format!(
            "Enunciado da Fase:\n{}\n\nCódigo do Aluno:\n```python\n{}\n```\n\
            Relatório AST: faltando={:?}, proibidos={:?}, hardcode={}, erro_sintaxe={:?}\n\
            Tag de Falha nos Testes: {:?} | Timeout de Loop Infinito: {} | Erro Runtime: {:?}",
            rules.problem_statement_md,
            student_code,
            ast_report.missing_required_nodes,
            ast_report.triggered_forbidden_nodes,
            ast_report.suspected_hardcode,
            ast_report.syntax_error_detail,
            failed_concept_tag,
            is_timeout_loop,
            runtime_error
        );

        let payload = json!({
            "systemInstruction": {
                "parts": [{ "text": system_prompt }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": user_context }]
            }],
            "generationConfig": {
                "temperature": 0.3,
                "responseMimeType": "application/json",
                "responseSchema": {
                    "type": "OBJECT",
                    "properties": {
                        "diagnostic_category": { "type": "STRING" },
                        "friendly_explanation": { "type": "STRING" },
                        "socratic_question": { "type": "STRING" },
                        "focus_line": { "type": "INTEGER" },
                        "conceptual_reminder": { "type": "STRING" }
                    },
                    "required": [
                        "diagnostic_category",
                        "friendly_explanation",
                        "socratic_question",
                        "focus_line",
                        "conceptual_reminder"
                    ]
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
            return Err(format!("HTTP Status {}", res.status()));
        }

        let body: Value = res.json().await.map_err(|e| e.to_string())?;
        let text_json = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Payload sem campo text")?;

        let parsed: Value = serde_json::from_str(text_json).map_err(|e| e.to_string())?;

        let diagnostic_category = parsed["diagnostic_category"]
            .as_str()
            .ok_or("Missing diagnostic_category")?
            .to_string();
        let friendly_explanation = parsed["friendly_explanation"]
            .as_str()
            .ok_or("Missing friendly_explanation")?
            .to_string();
        let socratic_question = parsed["socratic_question"]
            .as_str()
            .ok_or("Missing socratic_question")?
            .to_string();

        Ok(SocraticHintResponse {
            diagnostic_category,
            friendly_explanation,
            socratic_question,
            focus_line: parsed["focus_line"].as_u64().unwrap_or(1).max(1) as usize,
            conceptual_reminder: parsed["conceptual_reminder"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            provider_used: model.to_string(),
        })
    }

    async fn call_ollama_local(
        &self,
        endpoint: &str,
        student_code: &str,
        rules: &LevelRuleConfig,
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
    ) -> Result<SocraticHintResponse, String> {
        let prompt = format!(
            "Responda APENAS um JSON válido com chaves diagnostic_category, friendly_explanation, socratic_question, focus_line, conceptual_reminder em Português sem nenhum código pronto.\n\
            Enunciado: {}\nCódigo:\n{}\nAST faltando={:?}, hardcode={}, timeout={}, tag={:?}",
            rules.problem_statement_md,
            student_code,
            ast_report.missing_required_nodes,
            ast_report.suspected_hardcode,
            is_timeout_loop,
            failed_concept_tag
        );

        let payload = json!({
            "model": self.ollama_model,
            "prompt": prompt,
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
        let text_json = body["response"].as_str().ok_or("Ollama response empty")?;
        let parsed: Value = serde_json::from_str(text_json).map_err(|e| e.to_string())?;

        Ok(SocraticHintResponse {
            diagnostic_category: parsed["diagnostic_category"]
                .as_str()
                .unwrap_or("LOGIC_MISMATCH")
                .to_string(),
            friendly_explanation: parsed["friendly_explanation"]
                .as_str()
                .ok_or("Missing friendly_explanation")?
                .to_string(),
            socratic_question: parsed["socratic_question"]
                .as_str()
                .ok_or("Missing socratic_question")?
                .to_string(),
            focus_line: parsed["focus_line"].as_u64().unwrap_or(1).max(1) as usize,
            conceptual_reminder: parsed["conceptual_reminder"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            provider_used: "LOCAL_OLLAMA".to_string(),
        })
    }

    /// Guardrail Determinístico: Remove qualquer bloco de código (> 1 linha ou bloco cercado)
    /// que o LLM tente vazar.
    pub fn sanitize_no_code_leak(text: &str) -> String {
        let code_block_re = Regex::new(r"(?s)```.*?```").unwrap();
        let cleaned = code_block_re.replace_all(text, "[trecho de código omitido pelo Mentor]");

        // Também remove sequências de 2+ linhas consecutivas que pareçam código Python indentado/def/while/for/return
        let multi_line_code_re =
            Regex::new(r"(?m)(?:^(?:def |while |for |if |elif |else:|return |    |\t).*$\r?\n){2,}")
                .unwrap();
        multi_line_code_re
            .replace_all(&cleaned, "[trecho de código omitido pelo Mentor]\n")
            .to_string()
    }

    /// Motor Socrático 100% Offline em Rust
    pub fn offline_socratic_fallback(
        ast_report: &AstInspectionReport,
        failed_concept_tag: Option<&str>,
        is_timeout_loop: bool,
    ) -> SocraticHintResponse {
        OfflineSocraticEngine::diagnose(ast_report, failed_concept_tag, is_timeout_loop, None)
    }
}
