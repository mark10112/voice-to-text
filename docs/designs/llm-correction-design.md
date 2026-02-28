# LLM Correction Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** LLM post-processing pipeline, prompt engineering, context management, domain detection

---

## 1. Pipeline Overview

```
                      ┌─────────────────────────┐
                      │    LLM Correction        │
  raw_text ──────────▶│                          │──────▶ corrected_text
                      │  ┌───────────────────┐   │
  context ───────────▶│  │  Prompt Builder    │   │
  (prev sentences,    │  │  (Thai-specific)   │   │
   domain,            │  └────────┬──────────┘   │
   user vocab)        │           │              │
                      │  ┌────────▼──────────┐   │
                      │  │  LLM Backend      │   │
                      │  │  (Ollama / llama_  │   │
                      │  │   cpp)            │   │
                      │  └───────────────────┘   │
                      └─────────────────────────┘
```

---

## 2. LLM Backend

### 2.1 Corrector Config

```rust
/// Configuration for the LLM corrector — provider-agnostic
pub struct LlmCorrectorConfig {
    pub provider: LlmProvider,
    pub base_url: String,           // "http://localhost:11434" or cloud URL
    pub api_key: Option<String>,    // None for local, required for cloud
    pub model: String,              // model name/id
    pub temperature: f32,           // 0.3
    pub timeout_secs: u64,          // 10
    pub target_language: String,    // "th", "en", etc. — drives prompt selection
}
```

### 2.2 Phase 1: Ollama REST API (MVP)

ง่ายที่สุด — user ติดตั้ง Ollama แยก แล้ว app เรียกผ่าน HTTP (Ollama native format)

```rust
pub struct LlmCorrector {
    config: LlmCorrectorConfig,
    prompt_builder: PromptBuilder,
    client: reqwest::Client,
}

impl LlmCorrector {
    pub fn from_config(config: LlmCorrectorConfig) -> Self {
        Self {
            prompt_builder: PromptBuilder::new(&config.target_language),
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn correct(
        &self,
        raw_text: &str,
        context: &CorrectionContext,
    ) -> Result<String> {
        match self.config.provider {
            LlmProvider::Ollama => self.correct_ollama(raw_text, context).await,
            LlmProvider::OpenAiCompatible => self.correct_openai(raw_text, context).await,
            _ => Ok(raw_text.to_string()), // Disabled / LlamaCpp handled elsewhere
        }
    }

    /// Ollama native API — POST /api/generate
    async fn correct_ollama(&self, raw_text: &str, context: &CorrectionContext) -> Result<String> {
        let prompt = self.prompt_builder.build(raw_text, context);

        let response = self.client
            .post(format!("{}/api/generate", self.config.base_url))
            .json(&serde_json::json!({
                "model": self.config.model,
                "prompt": prompt,
                "stream": false,
                "options": {
                    "temperature": self.config.temperature,
                    "top_p": 0.9,
                    "num_predict": 256,
                    "stop": ["\n\n", "---"]
                }
            }))
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        Ok(body["response"].as_str().unwrap_or(raw_text).trim().to_string())
    }

    /// OpenAI-compatible API — POST /v1/chat/completions
    /// Covers: OpenAI, Groq, Together.ai, LM Studio, vLLM, Ollama (OpenAI mode)
    async fn correct_openai(&self, raw_text: &str, context: &CorrectionContext) -> Result<String> {
        let (system_msg, user_msg) = self.prompt_builder.build_chat(raw_text, context);

        let mut req = self.client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .json(&serde_json::json!({
                "model": self.config.model,
                "messages": [
                    {"role": "system", "content": system_msg},
                    {"role": "user",   "content": user_msg}
                ],
                "temperature": self.config.temperature,
                "max_tokens": 256
            }))
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs));

        if let Some(key) = &self.config.api_key {
            req = req.bearer_auth(key);
        }

        let body: serde_json::Value = req.send().await?.json().await?;
        let corrected = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or(raw_text)
            .trim()
            .to_string();

        Ok(corrected)
    }

    /// Health check — works for both Ollama and OpenAI-compatible
    pub async fn health_check(&self) -> bool {
        let url = match self.config.provider {
            LlmProvider::Ollama => format!("{}/api/tags", self.config.base_url),
            LlmProvider::OpenAiCompatible => format!("{}/v1/models", self.config.base_url),
            _ => return false,
        };

        let mut req = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(2));

        if let Some(key) = &self.config.api_key {
            req = req.bearer_auth(key);
        }

        req.send().await.is_ok()
    }
}
```

### 2.3 Phase 2: llama_cpp In-Process (Optional)

ไม่ต้องพึ่ง Ollama — load model เข้า process เดียวกัน

```rust
pub struct LlamaCppCorrector {
    // ใช้ llama_cpp หรือ llama-cpp-2 crate
    // จะ implement เมื่อ HTTP approach ทำงานได้แล้ว
}
```

### 2.4 Default Model Recommendations

| Provider | Model | ขนาด (GGUF Q4) | RAM | Thai Quality | แนะนำ |
|----------|-------|----------------|-----|-------------|-------|
| Ollama | Qwen2.5-1.5B | 1.1 GB | ~2.5 GB | ดี | Low-end PC |
| **Ollama** | **Qwen2.5-3B** | **2.0 GB** | **~4 GB** | **ดีมาก** | **Default ✅** |
| Ollama | Typhoon2-Qwen2.5-7B | 4.7 GB | ~7.5 GB | ดีที่สุด (Thai) | GPU users |
| OpenAI API | gpt-4o-mini | cloud | - | ดีมาก (multilingual) | Cloud users |
| Groq | llama-3.3-70b | cloud | - | ดี (multilingual) | Fast cloud |

---

## 3. Prompt Engineering

### 3.1 Prompt Structure

```
┌─────────────────────────────────┐
│  System Instruction             │  ~100 tokens (fixed)
│  (Thai STT correction rules)    │
├─────────────────────────────────┤
│  Domain Hint                    │  ~10 tokens (if detected)
│  (Medical / Legal / Tech)       │
├─────────────────────────────────┤
│  User Vocabulary                │  ~50 tokens (top 5 entries)
│  (error → correction pairs)    │
├─────────────────────────────────┤
│  Few-Shot Examples              │  ~100 tokens (2-3 examples)
│  (Thai-specific corrections)    │
├─────────────────────────────────┤
│  Previous Context               │  ~150 tokens (2-3 sentences)
│  (rolling window)               │
├─────────────────────────────────┤
│  Current STT Output             │  ~100 tokens (raw_text)
│  (to be corrected)              │
├─────────────────────────────────┤
│  Instruction: "Corrected:"      │  ~5 tokens
└─────────────────────────────────┘
  Total: ~500 tokens input → ~100 tokens output
```

### 3.2 Prompt Builder

`PromptBuilder` is language-aware — it selects the correct system instruction and few-shot examples based on `target_language`.

```rust
pub struct PromptBuilder {
    language: String, // "th", "en", "zh", etc.
}

impl PromptBuilder {
    pub fn new(language: &str) -> Self {
        Self { language: language.to_string() }
    }

    /// Build flat prompt (for Ollama native API)
    pub fn build(&self, raw_text: &str, ctx: &CorrectionContext) -> String {
        let (system, _) = self.system_instruction();
        let examples = self.few_shot_examples();
        let mut prompt = String::with_capacity(2048);

        prompt.push_str(&system);
        self.append_context_parts(&mut prompt, ctx, raw_text, &examples);
        prompt
    }

    /// Build chat messages (for OpenAI-compatible API)
    pub fn build_chat(&self, raw_text: &str, ctx: &CorrectionContext) -> (String, String) {
        let (system_msg, _) = self.system_instruction();
        let examples = self.few_shot_examples();
        let mut user_msg = String::with_capacity(1024);
        self.append_context_parts(&mut user_msg, ctx, raw_text, &examples);
        (system_msg, user_msg)
    }

    fn append_context_parts(
        &self,
        buf: &mut String,
        ctx: &CorrectionContext,
        raw_text: &str,
        examples: &str,
    ) {
        // 1. Domain hint
        if let Some(domain) = &ctx.domain {
            buf.push_str(&format!("\nDomain: {}\n", domain));
        }
        // 2. User vocabulary
        if !ctx.user_vocab.is_empty() {
            buf.push_str("\nUser-specific terms:\n");
            for (error, correct) in ctx.user_vocab.iter().take(5) {
                buf.push_str(&format!("- \"{}\" → \"{}\"\n", error, correct));
            }
        }
        // 3. Few-shot examples
        buf.push_str(examples);
        // 4. Previous context
        if !ctx.previous_sentences.is_empty() {
            buf.push_str("\nPrevious context:\n");
            for sent in ctx.previous_sentences.iter().rev().take(3) {
                buf.push_str(&format!("- {}\n", sent));
            }
        }
        // 5. Current input
        buf.push_str(&format!("\nOriginal STT output:\n{}\n\nCorrected:\n", raw_text));
    }

    fn system_instruction(&self) -> (String, ()) {
        let text = match self.language.as_str() {
            "th" => SYSTEM_INSTRUCTION_TH,
            "en" => SYSTEM_INSTRUCTION_EN,
            _    => SYSTEM_INSTRUCTION_EN, // fallback to English generic
        };
        (text.to_string(), ())
    }

    fn few_shot_examples(&self) -> &'static str {
        match self.language.as_str() {
            "th" => FEW_SHOT_EXAMPLES_TH,
            "en" => FEW_SHOT_EXAMPLES_EN,
            _    => FEW_SHOT_EXAMPLES_EN,
        }
    }
}
```

### 3.3 System Instructions (per language)

```rust
/// Thai — focuses on tone marks, homophones, filler words, Thai punctuation
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

/// Generic English / multilingual — handles filler words, punctuation, common STT errors
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
```

### 3.4 Few-Shot Examples (per language)

```rust
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
```

---

## 4. Context Manager

### 4.1 Structure

```rust
pub struct CorrectionContext {
    pub previous_sentences: Vec<String>,   // rolling window (max 3)
    pub domain: Option<String>,            // detected domain
    pub user_vocab: Vec<(String, String)>, // error → correction
}

pub struct ContextManager {
    sentences: VecDeque<String>,
    max_sentences: usize,                  // default: 3
    domain_detector: DomainDetector,
    user_vocab: UserVocabulary,
    last_activity: Instant,
    silence_reset: Duration,               // default: 120s
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            sentences: VecDeque::with_capacity(5),
            max_sentences: 3,
            domain_detector: DomainDetector::new(),
            user_vocab: UserVocabulary::load_or_default(),
            last_activity: Instant::now(),
            silence_reset: Duration::from_secs(120),
        }
    }

    /// สร้าง context สำหรับส่งให้ LLM
    pub fn build_context(&self) -> CorrectionContext {
        let all_text = self.sentences.iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");

        CorrectionContext {
            previous_sentences: self.sentences.iter().cloned().collect(),
            domain: self.domain_detector.detect(&all_text),
            user_vocab: self.user_vocab.top_entries(5),
        }
    }

    /// เพิ่มประโยคใหม่เข้า context
    pub fn push_sentence(&mut self, sentence: String) {
        // ถ้า silence นานเกิน → reset context
        if self.last_activity.elapsed() > self.silence_reset {
            self.sentences.clear();
        }

        self.sentences.push_back(sentence);
        while self.sentences.len() > self.max_sentences {
            self.sentences.pop_front();
        }

        self.last_activity = Instant::now();
    }

    /// Reset context (user เปลี่ยน topic)
    pub fn reset(&mut self) {
        self.sentences.clear();
    }
}
```

### 4.2 Context Modes

| Mode | Context | Domain | User Vocab | Use Case |
|------|---------|--------|-----------|----------|
| **Fast** | ไม่ใช้ LLM | - | - | Quick notes, casual |
| **Standard** | ไม่มี context | ไม่ detect | ไม่ใช้ | General purpose |
| **Context** | 3 ประโยค | Auto-detect | ใช้ | Dictation, formal writing |

---

## 5. Domain Detection

### 5.1 Keyword-Based Detection

```rust
pub struct DomainDetector {
    domains: Vec<DomainConfig>,
}

struct DomainConfig {
    name: String,
    keywords: Vec<String>,
    threshold: usize, // minimum keyword matches
}

impl DomainDetector {
    pub fn new() -> Self {
        Self {
            domains: vec![
                DomainConfig {
                    name: "medical".into(),
                    keywords: vec![
                        "ผู้ป่วย", "ยา", "อาการ", "โรค", "แพทย์",
                        "วินิจฉัย", "โรงพยาบาล", "เบาหวาน", "ความดัน",
                    ].into_iter().map(String::from).collect(),
                    threshold: 2,
                },
                DomainConfig {
                    name: "legal".into(),
                    keywords: vec![
                        "กฎหมาย", "สัญญา", "ศาล", "จำเลย", "โจทก์",
                        "คดี", "ข้อพิพาท", "พระราชบัญญัติ",
                    ].into_iter().map(String::from).collect(),
                    threshold: 2,
                },
                DomainConfig {
                    name: "technical".into(),
                    keywords: vec![
                        "code", "function", "server", "deploy", "database",
                        "API", "bug", "ซอฟต์แวร์", "ระบบ",
                    ].into_iter().map(String::from).collect(),
                    threshold: 2,
                },
            ],
        }
    }

    pub fn detect(&self, text: &str) -> Option<String> {
        self.domains.iter()
            .filter_map(|d| {
                let count = d.keywords.iter()
                    .filter(|kw| text.contains(kw.as_str()))
                    .count();
                if count >= d.threshold {
                    Some((d.name.clone(), count))
                } else {
                    None
                }
            })
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name)
    }
}
```

---

## 6. User Vocabulary

### 6.1 Persistence

```rust
pub struct UserVocabulary {
    entries: Vec<VocabEntry>,
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct VocabEntry {
    pub error: String,
    pub correction: String,
    pub frequency: u32,
}

impl UserVocabulary {
    pub fn load_or_default() -> Self {
        let path = Self::vocab_path();
        let entries = if path.exists() {
            let data = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self { entries, path }
    }

    pub fn add(&mut self, error: String, correction: String) {
        if let Some(entry) = self.entries.iter_mut()
            .find(|e| e.error == error) {
            entry.correction = correction;
            entry.frequency += 1;
        } else {
            self.entries.push(VocabEntry {
                error, correction, frequency: 1,
            });
        }
        self.save();
    }

    pub fn top_entries(&self, n: usize) -> Vec<(String, String)> {
        let mut sorted = self.entries.clone();
        sorted.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        sorted.into_iter()
            .take(n)
            .map(|e| (e.error, e.correction))
            .collect()
    }

    fn save(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(&self.path, data);
        }
    }

    fn vocab_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("thai-vtt")
            .join("user-vocab.json")
    }
}
```

### 6.2 Storage Location

```
Windows: %APPDATA%\thai-vtt\user-vocab.json
macOS:   ~/Library/Application Support/thai-vtt/user-vocab.json
Linux:   ~/.config/thai-vtt/user-vocab.json
```

---

## 7. Correction Quality Evaluation

### 7.1 CER (Character Error Rate) — Primary Metric

```rust
/// คำนวณ CER สำหรับประเมินคุณภาพ
pub fn compute_cer(reference: &str, hypothesis: &str) -> f32 {
    let ref_chars: Vec<char> = reference.chars().collect();
    let hyp_chars: Vec<char> = hypothesis.chars().collect();

    let distance = edit_distance(&ref_chars, &hyp_chars);
    if ref_chars.is_empty() {
        return if hyp_chars.is_empty() { 0.0 } else { 1.0 };
    }

    distance as f32 / ref_chars.len() as f32
}
```

### 7.2 Expected Improvements

| Stage | CER (typical) | Improvement |
|-------|--------------|-------------|
| STT only (Whisper Medium) | ~10-15% | baseline |
| + LLM zero-shot | ~7-10% | 30-40% |
| + few-shot examples | ~5-8% | 45-55% |
| + context window | ~4-7% | 50-60% |

> อ้างอิง: HyPoradise (NeurIPS 2023), Whispering LLaMA (EMNLP 2023)

---

## 8. Fallback Strategy

```
LLM Available?
  │
  ├── Yes → Standard/Context correction
  │
  └── No (Ollama down / timeout / error)
       │
       ├── Use raw STT output (Fast Mode fallback)
       ├── Show warning icon in UI
       └── Retry connection every 30s in background
```

---

## 9. Dependencies

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }  # Ollama API calls
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "6.0"
tokio = { version = "1", features = ["full"] }

# Phase 2 (optional, in-process LLM):
# llama_cpp = "0.3"
```
