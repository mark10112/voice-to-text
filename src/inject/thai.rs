//! Thai-text validation utilities.
//!
//! Before injecting text produced by STT/LLM we validate that:
//!
//! 1. The string is **non-empty**.
//! 2. It contains **at least one Thai character** (U+0E01 – U+0E5B).
//! 3. Every character is either **ASCII, Thai, or whitespace** — no stray
//!    code points from other scripts that would indicate a transcription error.

use super::InjectError;

// ---------------------------------------------------------------------------
// Unicode ranges
// ---------------------------------------------------------------------------

/// First codepoint of the Thai Unicode block used for voice output.
///
/// U+0E00 is unassigned; the first real Thai character (ก) is U+0E01.
const THAI_START: char = '\u{0E01}';

/// Last codepoint covered by our validation range.
///
/// U+0E5B (๛ — Thai character khomut, end-of-text mark) is the last assigned
/// codepoint in the core Thai block we care about.
const THAI_END: char = '\u{0E5B}';

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Validate that `text` is suitable for Thai voice-to-text injection.
///
/// # Rules
///
/// | # | Rule | Example violation |
/// |---|------|-------------------|
/// | 1 | Non-empty | `""` |
/// | 2 | Contains ≥ 1 Thai char (U+0E01–U+0E5B) | `"hello world"` |
/// | 3 | Every char is ASCII, Thai, or whitespace | `"สวัสดี 你好"` |
///
/// # Errors
///
/// Returns [`InjectError::Validation`] with a message describing the first
/// rule that fails.
///
/// # Examples
///
/// ```
/// use voice_to_text::inject::{validate_thai_text, InjectError};
///
/// assert!(validate_thai_text("สวัสดีครับ").is_ok());
/// assert!(validate_thai_text("").is_err());
/// assert!(validate_thai_text("hello").is_err());
/// ```
pub fn validate_thai_text(text: &str) -> Result<(), InjectError> {
    // Rule 1: non-empty
    if text.is_empty() {
        return Err(InjectError::Validation("text must not be empty".into()));
    }

    // Rule 2: must contain at least one Thai character
    let has_thai = text.chars().any(is_thai);
    if !has_thai {
        return Err(InjectError::Validation(
            "text must contain at least one Thai character (U+0E01–U+0E5B)".into(),
        ));
    }

    // Rule 3: all characters must be ASCII, Thai, or whitespace
    let bad = text
        .chars()
        .find(|&c| !c.is_ascii() && !is_thai(c) && !c.is_whitespace());
    if let Some(invalid_char) = bad {
        return Err(InjectError::Validation(format!(
            "text contains unsupported character U+{:04X} ({})",
            invalid_char as u32, invalid_char,
        )));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns `true` if `c` falls within the Thai Unicode block (U+0E01–U+0E5B).
#[inline]
fn is_thai(c: char) -> bool {
    (THAI_START..=THAI_END).contains(&c)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Rule 1: non-empty --------------------------------------------------

    #[test]
    fn empty_string_returns_err() {
        let err = validate_thai_text("").unwrap_err();
        assert!(err.to_string().contains("empty"), "unexpected message: {err}");
    }

    // --- Rule 2: must contain Thai characters --------------------------------

    #[test]
    fn ascii_only_returns_err() {
        assert!(validate_thai_text("hello world").is_err());
    }

    #[test]
    fn numbers_only_returns_err() {
        assert!(validate_thai_text("12345").is_err());
    }

    #[test]
    fn whitespace_only_returns_err() {
        // Whitespace has no Thai characters → fails rule 2
        assert!(validate_thai_text("   ").is_err());
    }

    // --- Rule 2 (pass): valid Thai text -------------------------------------

    #[test]
    fn thai_consonant_only_returns_ok() {
        // Single Thai consonant "ก" (U+0E01)
        assert!(validate_thai_text("ก").is_ok());
    }

    #[test]
    fn thai_word_returns_ok() {
        // "สวัสดี" — hello
        assert!(validate_thai_text("สวัสดี").is_ok());
    }

    #[test]
    fn thai_with_ascii_punctuation_returns_ok() {
        // Mixed Thai + ASCII
        assert!(validate_thai_text("ก่า test 123").is_ok());
    }

    #[test]
    fn thai_with_whitespace_returns_ok() {
        assert!(validate_thai_text("ก ข ค").is_ok());
    }

    #[test]
    fn thai_with_newline_returns_ok() {
        assert!(validate_thai_text("ก\nข").is_ok());
    }

    #[test]
    fn thai_digits_are_valid() {
        // Thai digits U+0E50–U+0E59 (๐–๙) + consonants
        assert!(validate_thai_text("๑๒๓ กขค").is_ok());
    }

    // --- Rule 3: no non-ASCII, non-Thai, non-whitespace chars ---------------

    #[test]
    fn chinese_chars_return_err() {
        // Chinese characters mixed with Thai should fail rule 3
        assert!(validate_thai_text("สวัสดี 你好").is_err());
    }

    #[test]
    fn arabic_chars_return_err() {
        assert!(validate_thai_text("สวัสดี مرحبا").is_err());
    }
}
