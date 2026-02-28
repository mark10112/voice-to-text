//! Prompt builder for Thai (and multilingual) STT correction.
//!
//! [`PromptBuilder`] constructs two kinds of prompts:
//! * **Flat** (`build`) — single string, for Ollama native `/api/generate`.
//! * **Chat** (`build_chat`) — `(system_msg, user_msg)` tuple for any
//!   OpenAI-compatible `/v1/chat/completions` endpoint.
//!
//! The language is selected at construction time; Thai (`"th"`) and English
//! (`"en"`) have dedicated instructions and few-shot examples.  Any other
//! language code falls back to the English instructions.

// ---------------------------------------------------------------------------
// System instructions
// ---------------------------------------------------------------------------

/// Thai — covers tone marks, homophones, filler words, Thai punctuation.
const SYSTEM_INSTRUCTION_TH: &str = "\
คุณคือระบบแก้ไขข้อความจาก Speech-to-Text สำหรับภาษาไทย
หน้าที่: แก้ไขข้อผิดพลาดจากการถอดเสียง โดยรักษาความหมายเดิม

กฎ:
1. แก้ไขวรรณยุกต์และคำพ้องเสียงที่ผิด
2. ลบคำอุทาน (เอ่อ, อ่า, อ่านะ, ครับ/ค่ะ ที่ไม่จำเป็น) ออก
3. เพิ่มเครื่องหมายวรรคตอนที่เหมาะสม
4. รักษาคำภาษาอังกฤษและศัพท์เทคนิค ไม่แปลงเป็นภาษาไทย
5. ตอบเฉพาะข้อความที่แก้ไขแล้ว ไม่ต้องอธิบาย
6. ถ้าข้อความถูกต้องแล้ว ให้ตอบข้อความเดิมกลับมา";

/// Generic English / multilingual — filler words, punctuation, common STT errors.
const SYSTEM_INSTRUCTION_EN: &str = "\
You are a Speech-to-Text post-correction assistant.
Task: Fix transcription errors while preserving the original meaning.

Rules:
1. Fix mis-transcribed words (homophones, wrong words that sound similar).
2. Remove filler words (um, uh, like, you know, etc.).
3. Add appropriate punctuation and capitalisation.
4. Preserve technical terms, proper nouns, and code snippets exactly.
5. Reply with ONLY the corrected text — no explanation.
6. If the text is already correct, return it unchanged.";

// ---------------------------------------------------------------------------
// Few-shot examples
// ---------------------------------------------------------------------------

const FEW_SHOT_EXAMPLES_TH: &str = "
Examples:
Input: \"เอ่อ ผม เสร็จ งาน แล้ว นะ ครับ จะ ส่ง ให้ พรุ่งนี้\"
Output: \"ผมเสร็จงานแล้ว จะส่งให้พรุ่งนี้\"

Input: \"ไฟล์ มัน ไม่ โหลด เพราะ network connection มัน drop\"
Output: \"ไฟล์ไม่โหลดเพราะ network connection drop\"

Input: \"อ่า ผู้ป่วย มี ความดัน สูง 140 ต่อ 90\"
Output: \"ผู้ป่วยมีความดันสูง 140/90\"
";

const FEW_SHOT_EXAMPLES_EN: &str = "
Examples:
Input: \"um I finished the report uh it should be ready by tomorrow\"
Output: \"I finished the report. It should be ready by tomorrow.\"

Input: \"the file won't load because the network connection like dropped\"
Output: \"The file won't load because the network connection dropped.\"

Input: \"the patient has hypertension one forty over ninety\"
Output: \"The patient has hypertension 140/90.\"
";

// ---------------------------------------------------------------------------
// PromptBuilder
// ---------------------------------------------------------------------------

/// Builds STT-correction prompts in either flat or chat-message format.
///
/// # Example
/// ```rust
/// use voice_to_text::llm::PromptBuilder;
///
/// let builder = PromptBuilder::new("th");
/// let (system, user) = builder.build_chat("เอ่อ สวัสดี ครับ", None);
/// assert!(system.contains("Speech-to-Text"));
/// ```
pub struct PromptBuilder {
    language: String,
}

impl PromptBuilder {
    /// Create a new builder for the given ISO-639-1 language code.
    ///
    /// Supported codes with dedicated instructions: `"th"`, `"en"`.
    /// Any other code falls back to English instructions.
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
        }
    }

    /// Build a **flat** prompt string (suitable for Ollama `/api/generate`).
    ///
    /// Structure (in order):
    /// 1. System instruction
    /// 2. Few-shot examples
    /// 3. Context (if provided) — domain, user vocab, previous sentences
    /// 4. Current STT output + "Corrected:" cue
    pub fn build(&self, raw: &str, context: Option<&str>) -> String {
        let mut prompt = String::with_capacity(2048);
        prompt.push_str(self.system_instruction());
        prompt.push_str(self.few_shot_examples());
        if let Some(ctx) = context {
            prompt.push('\n');
            prompt.push_str(ctx);
        }
        prompt.push_str(&format!(
            "\nOriginal STT output:\n{}\n\nCorrected:\n",
            raw
        ));
        prompt
    }

    /// Build a **(system_msg, user_msg)** pair (for OpenAI-compatible APIs).
    ///
    /// * `system_msg` — the Thai/English system instruction.
    /// * `user_msg` — few-shot examples + optional context + raw STT input.
    pub fn build_chat(&self, raw: &str, context: Option<&str>) -> (String, String) {
        let system_msg = self.system_instruction().to_string();

        let mut user_msg = String::with_capacity(1024);
        user_msg.push_str(self.few_shot_examples());
        if let Some(ctx) = context {
            user_msg.push('\n');
            user_msg.push_str(ctx);
        }
        user_msg.push_str(&format!(
            "\nOriginal STT output:\n{}\n\nCorrected:\n",
            raw
        ));

        (system_msg, user_msg)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn system_instruction(&self) -> &'static str {
        match self.language.as_str() {
            "th" => SYSTEM_INSTRUCTION_TH,
            "en" => SYSTEM_INSTRUCTION_EN,
            _ => SYSTEM_INSTRUCTION_EN,
        }
    }

    fn few_shot_examples(&self) -> &'static str {
        match self.language.as_str() {
            "th" => FEW_SHOT_EXAMPLES_TH,
            "en" => FEW_SHOT_EXAMPLES_EN,
            _ => FEW_SHOT_EXAMPLES_EN,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Thai prompt tests
    // -----------------------------------------------------------------------

    #[test]
    fn thai_system_instruction_contains_language_rule() {
        let builder = PromptBuilder::new("th");
        let (system, _) = builder.build_chat("เอ่อ สวัสดี ครับ", None);

        assert!(
            system.contains("Speech-to-Text"),
            "system msg must mention Speech-to-Text"
        );
        assert!(
            system.contains("ภาษาไทย"),
            "system msg must mention Thai language"
        );
        assert!(
            system.contains("วรรณยุกต์"),
            "system msg must mention tone mark correction"
        );
        assert!(
            system.contains("อุทาน"),
            "system msg must mention filler word removal"
        );
        assert!(
            system.contains("เครื่องหมายวรรคตอน"),
            "system msg must mention punctuation"
        );
    }

    #[test]
    fn thai_user_msg_contains_few_shot_examples() {
        let builder = PromptBuilder::new("th");
        let (_, user) = builder.build_chat("ทดสอบ", None);

        assert!(user.contains("Examples:"), "user msg must contain Examples");
        assert!(
            user.contains("ผมเสร็จงานแล้ว"),
            "user msg must contain Thai few-shot output"
        );
    }

    #[test]
    fn thai_prompt_includes_raw_text_and_cue() {
        let builder = PromptBuilder::new("th");
        let raw = "เอ่อ ผม ทำงาน เสร็จ แล้ว ครับ";
        let (_, user) = builder.build_chat(raw, None);

        assert!(
            user.contains(raw),
            "user msg must contain the raw STT output"
        );
        assert!(
            user.contains("Original STT output:"),
            "user msg must have the 'Original STT output:' label"
        );
        assert!(
            user.contains("Corrected:"),
            "user msg must have the 'Corrected:' cue"
        );
    }

    #[test]
    fn thai_prompt_embeds_context_string() {
        let builder = PromptBuilder::new("th");
        let ctx = "Domain: Medical\nPrevious context:\n- ผู้ป่วยมีไข้สูง\n";
        let (_, user) = builder.build_chat("ประโยคใหม่", Some(ctx));

        assert!(
            user.contains("ผู้ป่วยมีไข้สูง"),
            "user msg must contain previous context sentences"
        );
        assert!(
            user.contains("Domain: Medical"),
            "user msg must contain domain hint"
        );
        assert!(
            user.contains("ประโยคใหม่"),
            "user msg must contain the raw text"
        );
    }

    #[test]
    fn no_context_produces_valid_prompt() {
        let builder = PromptBuilder::new("th");
        let (system, user) = builder.build_chat("สวัสดี", None);

        assert!(!system.is_empty());
        assert!(!user.is_empty());
        assert!(user.contains("สวัสดี"));
    }

    // -----------------------------------------------------------------------
    // English prompt tests
    // -----------------------------------------------------------------------

    #[test]
    fn english_system_instruction_mentions_filler_words() {
        let builder = PromptBuilder::new("en");
        let (system, _) = builder.build_chat("um hello there", None);

        assert!(system.contains("Speech-to-Text"));
        assert!(
            system.contains("filler words"),
            "English system msg must mention filler words"
        );
        assert!(
            system.contains("punctuation"),
            "English system msg must mention punctuation"
        );
    }

    #[test]
    fn english_user_msg_has_few_shot_examples() {
        let builder = PromptBuilder::new("en");
        let (_, user) = builder.build_chat("um test", None);

        assert!(user.contains("Examples:"));
        assert!(user.contains("I finished the report."));
    }

    // -----------------------------------------------------------------------
    // Unknown language falls back to English
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_language_falls_back_to_english() {
        let builder = PromptBuilder::new("ja");
        let (system, user) = builder.build_chat("test", None);

        // Should use English instructions
        assert!(system.contains("Speech-to-Text"));
        assert!(user.contains("I finished the report."));
    }

    // -----------------------------------------------------------------------
    // Flat prompt (build)
    // -----------------------------------------------------------------------

    #[test]
    fn flat_prompt_contains_all_sections() {
        let builder = PromptBuilder::new("th");
        let prompt = builder.build("สวัสดี", None);

        assert!(prompt.contains("Speech-to-Text"));
        assert!(prompt.contains("Examples:"));
        assert!(prompt.contains("สวัสดี"));
        assert!(prompt.contains("Corrected:"));
    }

    #[test]
    fn flat_prompt_with_context() {
        let builder = PromptBuilder::new("th");
        let ctx = "Domain: Technical\n";
        let prompt = builder.build("บั๊กมันอยู่ที่ไหน", Some(ctx));

        assert!(prompt.contains("Domain: Technical"));
        assert!(prompt.contains("บั๊กมันอยู่ที่ไหน"));
    }
}
