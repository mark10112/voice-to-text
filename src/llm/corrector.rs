//! Core `LlmCorrector` trait and `ApiCorrector` implementation.
//!
//! `ApiCorrector` calls any OpenAI-compatible `/v1/chat/completions` endpoint
//! — Ollama (OpenAI mode), OpenAI, Groq, LM Studio, vLLM, etc.
//! All connection details come from [`LlmConfig`]; nothing is hardcoded.

use async_trait::async_trait;
use thiserror::Error;

use crate::config::LlmConfig;
use crate::llm::prompt::PromptBuilder;

// ---------------------------------------------------------------------------
// LlmError
// ---------------------------------------------------------------------------

/// Errors that can occur during LLM correction.
#[derive(Debug, Error)]
pub enum LlmError {
    /// HTTP transport or connection error.
    #[error("HTTP request failed: {0}")]
    Request(String),

    /// The request did not complete within the configured timeout.
    #[error("LLM request timed out")]
    Timeout,

    /// The HTTP response could not be parsed as expected JSON.
    #[error("failed to parse LLM response: {0}")]
    Parse(String),

    /// The LLM returned a response with no usable text content.
    #[error("LLM returned an empty response")]
    EmptyResponse,
}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            LlmError::Timeout
        } else {
            LlmError::Request(e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// LlmCorrector trait
// ---------------------------------------------------------------------------

/// Async trait for LLM-based text correction.
///
/// Implementors must be `Send + Sync` so they can be shared across threads
/// (e.g. wrapped in `Arc<dyn LlmCorrector>`).
///
/// # Arguments
/// * `raw`     – Raw STT transcript to correct.
/// * `context` – Optional pre-built context string from [`ContextManager`]
///               (domain hint, user vocab, previous sentences).
#[async_trait]
pub trait LlmCorrector: Send + Sync {
    async fn correct(&self, raw: &str, context: Option<&str>) -> Result<String, LlmError>;
}

// ---------------------------------------------------------------------------
// ApiCorrector
// ---------------------------------------------------------------------------

/// Calls an OpenAI-compatible `/v1/chat/completions` endpoint.
///
/// Works with: Ollama (OpenAI mode), OpenAI, Groq, Together.ai, LM Studio,
/// vLLM — any provider that speaks the OpenAI chat-completions wire format.
///
/// # No hardcoded URLs
/// All connection details (`base_url`, `api_key`, `model`) come exclusively
/// from the [`LlmConfig`] passed to [`ApiCorrector::from_config`].
pub struct ApiCorrector {
    client: reqwest::Client,
    config: LlmConfig,
    prompt_builder: PromptBuilder,
}

impl ApiCorrector {
    /// Build an `ApiCorrector` from application config.
    ///
    /// The HTTP client is pre-configured with the per-request timeout from
    /// `config.timeout_secs`.  A default (no-timeout) client is used as a
    /// last-resort fallback if the builder fails (should never happen in
    /// practice).
    pub fn from_config(config: &LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // Default to Thai; the language could be made configurable later.
        let prompt_builder = PromptBuilder::new("th");

        Self {
            client,
            config: config.clone(),
            prompt_builder,
        }
    }
}

#[async_trait]
impl LlmCorrector for ApiCorrector {
    /// Send `raw` to the configured OpenAI-compatible endpoint for correction.
    ///
    /// The `Authorization: Bearer …` header is attached **only** when
    /// `config.api_key` is `Some(key)` and `key` is non-empty — safe for
    /// Ollama and other local providers that require no authentication.
    async fn correct(&self, raw: &str, context: Option<&str>) -> Result<String, LlmError> {
        let (system_msg, user_msg) = self.prompt_builder.build_chat(raw, context);

        let url = format!("{}/v1/chat/completions", self.config.base_url);

        let body = serde_json::json!({
            "model":       self.config.model,
            "messages": [
                { "role": "system", "content": system_msg },
                { "role": "user",   "content": user_msg   }
            ],
            "stream":      false,
            "temperature": self.config.temperature,
            "max_tokens":  256
        });

        let mut req = self.client.post(&url).json(&body);

        // Attach Authorization header only when api_key is a non-empty string.
        let key = self.config.api_key.as_deref().unwrap_or("");
        if !key.is_empty() {
            req = req.bearer_auth(key);
        }

        let response = req.send().await?;

        let json: serde_json::Value = response.json().await.map_err(|e| {
            LlmError::Parse(e.to_string())
        })?;

        let corrected = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(LlmError::EmptyResponse)?
            .trim()
            .to_string();

        if corrected.is_empty() {
            return Err(LlmError::EmptyResponse);
        }

        Ok(corrected)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LlmConfig, LlmProvider};

    fn make_config(api_key: Option<&str>) -> LlmConfig {
        LlmConfig {
            enabled: true,
            provider: LlmProvider::OpenAiCompatible,
            base_url: "http://localhost:11434".into(),
            api_key: api_key.map(|s| s.to_string()),
            model: "qwen2.5:3b".into(),
            temperature: 0.3,
            timeout_secs: 10,
        }
    }

    #[test]
    fn from_config_builds_without_panic() {
        let config = make_config(None);
        let _corrector = ApiCorrector::from_config(&config);
    }

    #[test]
    fn from_config_accepts_empty_api_key() {
        let config = make_config(Some(""));
        let _corrector = ApiCorrector::from_config(&config);
    }

    #[test]
    fn from_config_accepts_real_api_key() {
        let config = make_config(Some("sk-test-1234"));
        let _corrector = ApiCorrector::from_config(&config);
    }

    /// Verify that `ApiCorrector` is object-safe (usable as `dyn LlmCorrector`).
    #[test]
    fn corrector_is_object_safe() {
        let config = make_config(None);
        let corrector: Box<dyn LlmCorrector> = Box::new(ApiCorrector::from_config(&config));
        // Just holding the trait object is sufficient to verify object-safety.
        drop(corrector);
    }
}
