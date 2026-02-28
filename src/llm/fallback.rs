//! Fallback corrector — wraps any [`LlmCorrector`] and returns raw text on error.
//!
//! When the underlying LLM call fails for any reason (`Request`, `Timeout`,
//! `Parse`, `EmptyResponse`) [`FallbackCorrector`] silently returns the
//! original raw STT text instead of propagating the error.  This keeps the
//! pipeline functional even when Ollama is not running or the API is
//! unreachable.

use async_trait::async_trait;

use crate::llm::corrector::{LlmCorrector, LlmError};

// ---------------------------------------------------------------------------
// FallbackCorrector
// ---------------------------------------------------------------------------

/// A transparent wrapper around any [`LlmCorrector`] that never returns an
/// error — on failure it returns `raw` unchanged.
///
/// # Example
/// ```rust
/// use voice_to_text::llm::{ApiCorrector, FallbackCorrector};
/// use voice_to_text::config::LlmConfig;
///
/// let inner = ApiCorrector::from_config(&LlmConfig::default());
/// let corrector = FallbackCorrector::new(inner);
/// // `corrector` now implements LlmCorrector and is safe to use even when
/// // the LLM backend is unavailable.
/// ```
pub struct FallbackCorrector<C: LlmCorrector> {
    inner: C,
}

impl<C: LlmCorrector> FallbackCorrector<C> {
    /// Wrap `inner` with fallback behaviour.
    pub fn new(inner: C) -> Self {
        Self { inner }
    }

    /// Return a reference to the wrapped corrector.
    pub fn inner(&self) -> &C {
        &self.inner
    }
}

#[async_trait]
impl<C: LlmCorrector + Send + Sync> LlmCorrector for FallbackCorrector<C> {
    /// Attempt LLM correction; return `raw` unchanged if any error occurs.
    ///
    /// This implementation **never** returns `Err(_)`.
    async fn correct(&self, raw: &str, context: Option<&str>) -> Result<String, LlmError> {
        match self.inner.correct(raw, context).await {
            Ok(corrected) => Ok(corrected),
            Err(_err) => {
                log::warn!(
                    "LLM correction failed — returning raw text (len={})",
                    raw.len()
                );
                Ok(raw.to_string())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // -----------------------------------------------------------------------
    // Test doubles
    // -----------------------------------------------------------------------

    /// Always succeeds with a fixed corrected string.
    struct AlwaysOk(String);

    #[async_trait]
    impl LlmCorrector for AlwaysOk {
        async fn correct(&self, _raw: &str, _ctx: Option<&str>) -> Result<String, LlmError> {
            Ok(self.0.clone())
        }
    }

    /// Always returns the given error.
    struct AlwaysFails(LlmErrorKind);

    enum LlmErrorKind {
        Request,
        Timeout,
        Parse,
        Empty,
    }

    #[async_trait]
    impl LlmCorrector for AlwaysFails {
        async fn correct(&self, _raw: &str, _ctx: Option<&str>) -> Result<String, LlmError> {
            let err = match self.0 {
                LlmErrorKind::Request => LlmError::Request("connection refused".into()),
                LlmErrorKind::Timeout => LlmError::Timeout,
                LlmErrorKind::Parse => LlmError::Parse("bad json".into()),
                LlmErrorKind::Empty => LlmError::EmptyResponse,
            };
            Err(err)
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn passes_through_success() {
        let corrector = FallbackCorrector::new(AlwaysOk("แก้ไขแล้ว".into()));
        let result = corrector.correct("ผิด", None).await.unwrap();
        assert_eq!(result, "แก้ไขแล้ว");
    }

    #[tokio::test]
    async fn returns_raw_on_request_error() {
        let corrector = FallbackCorrector::new(AlwaysFails(LlmErrorKind::Request));
        let result = corrector.correct("ข้อความเดิม", None).await.unwrap();
        assert_eq!(result, "ข้อความเดิม");
    }

    #[tokio::test]
    async fn returns_raw_on_timeout() {
        let corrector = FallbackCorrector::new(AlwaysFails(LlmErrorKind::Timeout));
        let result = corrector.correct("ข้อความเดิม", None).await.unwrap();
        assert_eq!(result, "ข้อความเดิม");
    }

    #[tokio::test]
    async fn returns_raw_on_parse_error() {
        let corrector = FallbackCorrector::new(AlwaysFails(LlmErrorKind::Parse));
        let result = corrector.correct("ข้อความเดิม", None).await.unwrap();
        assert_eq!(result, "ข้อความเดิม");
    }

    #[tokio::test]
    async fn returns_raw_on_empty_response() {
        let corrector = FallbackCorrector::new(AlwaysFails(LlmErrorKind::Empty));
        let result = corrector.correct("ข้อความเดิม", None).await.unwrap();
        assert_eq!(result, "ข้อความเดิม");
    }

    #[tokio::test]
    async fn never_returns_err() {
        let corrector = FallbackCorrector::new(AlwaysFails(LlmErrorKind::Timeout));
        // Must always be Ok(_), even on failure
        assert!(corrector.correct("test", None).await.is_ok());
    }

    /// FallbackCorrector<C> must itself be a valid LlmCorrector (object-safe).
    #[test]
    fn fallback_is_object_safe() {
        let inner = AlwaysOk("ok".into());
        let _: Box<dyn LlmCorrector> = Box::new(FallbackCorrector::new(inner));
    }
}
