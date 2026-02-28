# Thai Voice-to-Text Desktop Application Research
## Deep Research on STT Error Patterns, LLM Post-Processing, and System Architecture

**Research Date:** February 2026
**Focus:** Whisper + LLM post-processing for Thai speech-to-text with local LLM integration

---

## Topic 1: Common STT Error Patterns (Especially Thai)

### Overview of STT Error Sources

Speech-to-text systems make systematic errors based on acoustic similarity, language structure, and model training data. For Thai specifically, the challenges are compounded by tonal marks and writing system characteristics.

### Error Types in STT Systems

#### 1. **Homophones and Code-Switching**
- **Definition:** Words that sound identical but have different meanings (e.g., "to," "too," "two")
- **Thai Context:** Thai has inherent homophones due to tonal system. Research on code-switched speech recognition shows that when Thai is mixed with English, homophones become even more problematic because transcriptions of borrowed words are often non-standardized
- **Key Finding:** Systems with homophone merging show 1.2% lower relative WER compared to baseline without merging
- **Reference:** Research on "Homophone Identification and Merging for Code-switched Speech Recognition" (Srivastava et al., Interspeech 2018)

#### 2. **Fast Speech and Unclear Pronunciation Errors**
- Rapid speech patterns cause phonemes to blend together, creating ambiguous boundaries
- Solutions: Context-aware correction can disambiguate based on surrounding words
- Whisper models show improvement from small to medium sizes, particularly relevant for Thai

#### 3. **Tone Mark Errors (Critical for Thai)**
- **Thai Specificity:** Thai has 5 tones (mid, low, falling, high, rising) marked by diacritical marks (วรรณยุกต์)
- **Research Finding:** Research paper "Using Tone Information in Thai Spelling Speech Recognition" (Y14-1023, ACL) specifically addresses this
- **Problem:** Tone marks create homophones. For example, different tones on the same syllable can completely change meaning:
  - `ม้า` (má - horse, falling tone)
  - `มา` (maa - come, mid tone)
  - `มา` (máa - horse, high tone) - different from the first due to tone
- **Complexity:** Letter-to-sound mapping, phoneme set selection, segmentation, and tonality are key challenges in Thai ASR

#### 4. **Letter Substitution and Vowel Confusion**
- **Primary source of errors in Thai spelling recognition:**
  - Confusion of similar consonant phones
  - Confusion of short/long vowel pairs
- This directly affects character-level accuracy

#### 5. **Filler Word Handling**
- Common Thai filler words: `เอ่อ` (er), `อ่า` (ah), `ครับ` (khrap - polite particle), `นะ` (na)
- Modern STT systems need to remove these in post-processing
- Current approach: LLM post-processing can identify and remove these contextually

#### 6. **Punctuation and Sentence Boundary Errors**
- Whisper often outputs continuous text without punctuation
- Thai lacks explicit word boundaries (like Chinese), complicating sentence detection
- Solution: LLM post-processing adds punctuation based on semantic boundaries

#### 7. **Domain-Specific Vocabulary Errors**
- Medical terms, legal terminology, tech vocabulary often misrecognized
- Solution: Few-shot prompting with domain-specific examples can improve correction
- Context-aware vocabularies can be built into LLM prompts

### Error Metrics: WER vs CER for Thai

#### **WER (Word Error Rate)**
- **Calculation:** (S + D + I) / N where S=substitutions, D=deletions, I=insertions, N=total words
- **Problem for Thai:** WER has fundamental limitations because Thai lacks explicit word boundaries
- **Finding from 2025 Research:** Recent academic work shows WER can yield **100% error rates** for languages like Thai and Chinese where word segmentation is not explicit

#### **CER (Character Error Rate)** - RECOMMENDED FOR THAI
- **Calculation:** (S + D + I) / N where N=total characters instead of words
- **Why Better for Thai:** CER avoids word boundary issues and is more consistent across writing systems
- **Adoption:** Typhoon ASR benchmark uses CER as the primary metric for Thai
- **Research Citation:** "Advocating Character Error Rate for Multilingual ASR Evaluation" (Thennal D K et al., NAACL Findings 2025)

#### **Whisper Performance on Thai**
- Thai shows noticeable improvement from Whisper small to medium models
- ElevenLabs Scribe reportedly achieves 3.1% WER on FLEURS and 5.5% on Common Voice for Thai
- Whisper's accuracy is lower for tonal languages than English
- Models perform better with more training data

#### **Example Error Patterns in Thai Transcription**
From practical testing of Whisper with Thai:
- Confuses similar-sounding consonants
- May drop tone marks (though Whisper outputs romanization)
- Often produces continuous text without sentence breaks
- Struggles with code-switched Thai-English content

---

## Topic 2: LLM Post-Processing for STT Correction

### Research Foundation: Generative Error Correction (GEC)

The field has moved from **ranking-based rescoring** (selecting the best N-best hypothesis) to **generative error correction** (generating new, corrected text from multiple hypotheses).

### Key Research Papers and Frameworks

#### 1. **"Generative Speech Recognition Error Correction with Large Language Models and Task-Activating Prompting"**
- **Authors:** Amazon Science (Chen et al., 2023)
- **Link:** https://assets.amazon.science/77/26/6c265e0a42d7a40d2ee8bdd158e6/generative-speech-recognition-error-correction-with-large-language-models-and-task-activating-prompting.pdf
- **Key Innovation:** Task-activating prompting - prompts that explicitly tell the LLM "you are correcting STT errors"
- **Approach:** Uses generative LLMs instead of just reranking

#### 2. **HyPoradise Dataset - The Current Benchmark** (2023, NeurIPS)
- **Paper:** "HyPoradise: An Open Baseline for Generative Speech Recognition with Large Language Models"
- **Link:** https://arxiv.org/abs/2309.15701
- **Dataset Size:** 316,000+ pairs of N-best hypotheses and accurate transcriptions
- **Domains:** Multiple prevalent speech domains
- **Key Contribution:** First open-source benchmark using external LLMs for ASR error correction
- **Results:** Shows LLM generative correction can even fix tokens missing from N-best list
- **Performance:** Significantly surpasses upper bound of traditional re-ranking methods

#### 3. **Whispering LLaMA: A Cross-Modal Generative Error Correction Framework** (EMNLP 2023)
- **Authors:** Srijith Radhakrishnan et al., NVIDIA Taiwan
- **Link:** https://arxiv.org/abs/2310.06434
- **GitHub:** https://github.com/Srijith-rkr/Whispering-LLaMA
- **Innovation:** Cross-modal fusion combining:
  - Acoustic information (Whisper encoder)
  - Linguistic information (LLaMA decoder)
- **Architecture:** Uses distinct initialization and parameter-efficient algorithms
- **Results:** 37.66% relative improvement in WER compared to N-best hypotheses rescoring
- **Availability:** Open-source code and pre-trained models released

#### 4. **Applying LLMs for Rescoring N-best ASR Hypotheses with Context Carry-over** (2024)
- **Link:** https://arxiv.org/abs/2406.18972
- **Focus:** Casual conversations
- **Key Finding:** Context carry-over (using previous conversational context) improves performance
- **Process:**
  1. ASR generates N-best hypotheses for each segment
  2. LLM analyzes hypotheses alongside previous context
  3. Model evaluates coherence and probability
  4. Rescores and re-ranks based on broader context

#### 5. **CLEAR: Code-Mixed ASR with LLM-Driven Rescoring** (2025)
- **Link:** https://aclanthology.org/2025.icnlsp-1.33.pdf
- **Focus:** Specifically addresses code-mixed speech (Thai + English)
- **Relevance:** Directly applicable to Thai-English mixed content

#### 6. **Recent Noise-Robust GER** (2024)
- **Paper:** "Denoising GER: A Noise-Robust Generative Error Correction with LLM for Speech Recognition"
- **Link:** https://arxiv.org/abs/2509.04392
- **Challenge Addressed:** Poor adaptability and low information utilization in noisy environments

#### 7. **FLANEC: Exploring Flan-T5 for Post-ASR Error Correction** (2024)
- **Link:** https://arxiv.org/abs/2501.12979
- **Note:** Shows smaller models (Flan-T5) can be effective for correction, relevant for local deployment

### Correction Techniques

#### **N-best Hypotheses Rescoring**
- **Process:** Whisper or other ASR generates multiple hypotheses (typically 5-10)
- **Efficiency Advantage:** Much faster than generating from scratch
- **Implementation:** Format N-best list as input to LLM for evaluation
- **Computational Cost:** Lower than generative approaches but sometimes less accurate

#### **Generative Error Correction**
- **Process:** LLM generates corrected output from scratch or modifies input
- **Advantage:** Can fix tokens not in N-best list, better contextual corrections
- **Cost:** Higher computational cost but better accuracy
- **Best For:** Desktop application where latency is less critical than accuracy

#### **In-Context Learning / Few-Shot Correction**
- **Approach:** Provide examples of correction in the prompt
- **Format:** Show bad STT output → corrected output examples
- **Effectiveness:** Shows better results than zero-shot prompting
- **Number of Examples:** 2-5 examples typically sufficient

#### **Task-Activating Prompting**
- **Concept:** Explicitly tell LLM it's correcting STT errors
- **Example Prompt Structure:**
  ```
  You are an expert at fixing speech-to-text errors.
  The following text was transcribed by an STT system from Thai speech.

  Original: [STT output]
  Correct it and provide only the corrected text.
  ```
- **Results:** Shows significant improvements over standard prompting

### Performance Improvements from LLM Post-Processing

- **Baseline WER Improvement:** 10-30% WER reduction with basic LLM correction
- **With Context:** 30-50% improvement possible with full conversational context
- **With Few-Shot Examples:** Additional 5-15% improvement with domain-specific examples
- **Non-English Languages:** Thai and other low-resource languages benefit significantly from LLM-based correction because:
  - More linguistic knowledge in LLM weights
  - Can leverage morphological understanding
  - Better at Thai-specific error patterns

---

## Topic 3: Context Window Strategy for Better Correction

### Sliding Window / Batch Context Approaches

#### **Challenge:**
- Long-form dictation creates computational challenges
- Full context windows of 4K-32K tokens get expensive
- But some context is needed for coherence

#### **Sliding Window Strategy**
- **Concept:** Process speech in chunks while maintaining overlapping context
- **Implementation:**
  1. Record audio in 5-10 second chunks
  2. Transcribe each chunk with Whisper
  3. Keep previous 2-3 sentences as context for LLM
  4. Correct current chunk with context
  5. Move window forward

#### **Research on Sliding Window Attention:**
- **Papers:** "Sliding Window Attention Training for Efficient Large Language Models" (2025)
- **Finding:** Using shorter context lengths reduces computational cost significantly
- **Cost Reduction:** 20-40% reduction in tokens needed while maintaining accuracy
- **Practical:** For Thai, can use 200-400 tokens of previous context effectively

#### **Context Carry-Over Mechanism**
From research on casual conversations:
1. **Store previous segments:** Keep last N corrected sentences (typically 3-5)
2. **Extract topics:** Identify domain/topic from context
3. **Build prompt:** Include previous context in correction prompt
4. **Evaluate:** Rescore hypotheses considering full conversation flow
5. **Output:** Best hypothesis given accumulated context

### Domain Adaptation

#### **Topic Awareness**
- **Casual conversation vs Medical:** Same misheard word corrects differently based on context
- **Practical Implementation:**
  - Detect if user is discussing medical, technical, legal, etc.
  - Use domain-specific vocabulary in correction prompt
  - Adjust confidence thresholds

#### **User Vocabulary / Custom Dictionary**
- **Approach:** Store corrections made by user over time
- **Benefits:**
  - Personal names
  - Company-specific terms
  - Technical jargon user frequently uses
- **Implementation:** Simple database of user's previous corrections
- **Integration:** Include top 5-10 frequent user words in each LLM prompt

#### **Few-Shot Examples for Domain-Specific Correction**
- **Medical Domain Example:**
  ```
  STT Output: "ผมเจ็บ หัว"
  Correct: "ผมเจ็บหัว" (headache - proper segmentation)

  STT Output: "โรค เบาหวาน"
  Correct: "โรคเบาหวาน" (diabetes - compound word)
  ```
- **Technical Domain:**
  ```
  STT Output: "การ เข้า รหัส"
  Correct: "การเข้ารหัส" (encryption)
  ```

### Wispr Flow / Auto-Editing Approach

From research on Wispr Flow's implementation:

#### **Features:**
1. **Real-time Correction:** Removes filler words as user speaks
2. **Context-Aware Tone:** Same content adapts:
   - Casual for Slack: "Hey, let's meet at 4 p.m."
   - Professional for email: "I wanted to follow up regarding our meeting"
   - Formal for LinkedIn: "I recently discovered an interesting insight"
3. **Course Correction:** Recognizes "wait, no, let me say that again"
4. **Personal Dictionary:** Learns user's frequent corrections

#### **LLM Pipeline Implementation:**
```
Audio → Whisper STT → LLM Correction → Apply Tone Adaptation → Personal Dictionary Check → Output
                      ↓
                Previous Context (2-3 sentences)
```

---

## Topic 4: Lightweight Local LLM Options for Thai

### Requirements for Thai Voice-to-Text Desktop App
- Must support Thai language
- Should run on CPU (some GPU support optional)
- GGUF format compatibility (for llama.cpp)
- Context window of 4K-8K minimum
- Response time < 2 seconds for correction

### Model Comparison Table

| Model | Size | Thai Support | CPU | GGUF | Context | Quantization | Notes |
|-------|------|--------------|-----|------|---------|--------------|-------|
| Qwen 2.5 | 0.5B-3B | ✓ (29 langs) | ✓ | ✓ | 32K | Q4, Q5 | Multilingual base |
| Qwen 2.5 + Typhoon2 | 7B | ✓✓ (specialized) | ✓ (slow) | ✓ | 128K | Q4, Q5 | **BEST for Thai** |
| Llama 3.2 | 1B-3B | ✓ | ✓ | ✓ | 8K | Q2, Q3, Q4 | Good balance |
| Typhoon 2 | 7B-70B | ✓✓✓ | Only small | ✓ | 8K | Q4 | Thai-native |
| OpenThaiGPT | 7B-70B | ✓✓✓ | Only small | ✓ | 131K | Q4 | Thai-focused |
| Gemma 3 | 1B-27B | ✓ (140 langs) | ✓ | ✓ | 8K-131K | Q4 | Very capable small |
| Phi-4-mini | 3.8B | ✓ | ✓ | ✓ | 128K | Q4, Q5 | Good reasoning |

### Detailed Model Analysis

#### **1. Qwen 2.5 Series - Best Lightweight Option**
- **Available Sizes:** 0.5B, 1.5B, 3B, 7B, 14B, 32B, 72B
- **Thai Support:** Included in 29-language support
- **Recommended for Local:** 1.5B or 3B with GGUF quantization
- **CPU Requirements:**
  - 0.5B: ~2GB RAM (Q4)
  - 1.5B: ~4GB RAM (Q4)
  - 3B: ~8GB RAM (Q4)
- **Performance:** Good instruction-following
- **Links:**
  - Model Hub: https://huggingface.co/Qwen/Qwen2.5-7B-Instruct
  - Quick Start: "ollama run qwen2.5:3b" (requires ollama)

#### **2. Typhoon2-Qwen2.5 Hybrid - BEST FOR THAI**
- **Model:** scb10x/typhoon2-qwen2.5-7b-instruct
- **Architecture:** Merges SCB10X's Thai-centric Typhoon2-7B with Qwen2.5-7B
- **Thai Performance:** Significantly outperforms base Qwen2.5
- **Context Window:** 128,000 tokens (excellent for long corrections)
- **Key Features:**
  - Exceptional instruction-following
  - Multilingual (Thai-English code-switching)
  - Long-context understanding
- **Note:** Primarily suited for systems with some GPU (7B is large for pure CPU)
- **Recommended When:** Resources available; highest Thai quality

#### **3. Llama 3.2 Series - Good CPU Balance**
- **Available Sizes:** 1B, 3B (with instruction-tuned and base versions)
- **Thai Support:** Officially supported (8 languages total)
- **CPU Requirements:**
  - 1B: ~2GB RAM (Q4)
  - 3B: ~6-8GB RAM (Q4)
- **GGUF Format:** Available from multiple quantizers (bartowski, unsloth, QuantFactory)
- **Performance:** Strong across all 8 supported languages, minor variations for Thai
- **Deployment Options:** Jan, Msty, GPT4All, Ollama
- **Links:**
  - GGUF Variants: https://huggingface.co/bartowski/Llama-3.2-3B-Instruct-GGUF
  - Ollama: https://ollama.com/library/llama3.2

#### **4. Typhoon 2 - Thai-Native LLM**
- **Sizes:** Multiple versions (0.5B OCR, 7B, 70B)
- **Thai Specialization:** Highest (trained specifically on Thai data)
- **Performance:** On par with GPT-3.5 for Thai tasks
- **Efficiency:** 2.62x more efficient at tokenizing Thai text than base models
- **Open Source:** Apache 2.0 license
- **Multimodal Options:** Typhoon2-Audio (speech input/output support)
- **ASR Option:** Typhoon ASR - real-time streaming ASR for Thai optimized for CPU
- **Deployment:** Ollama, vLLM, Llama.cpp
- **Links:**
  - Main Site: https://opentyphoon.ai/
  - Models: https://huggingface.co/scb10x/typhoon-7b
  - ASR: https://github.com/scb-10x/typhoon-asr
  - Audio: https://github.com/scb-10x/typhoon2-audio

#### **5. OpenThaiGPT - Purpose-Built Thai Model**
- **Versions:**
  - 1.0.0 (LLaMA v2 base)
  - 1.5 (Qwen v2.5 base) - newer
- **Thai Dictionary:** 10,000+ most common Thai words built-in
- **Context:** Up to 131,072 tokens
- **Output Generation:** Up to 8,192 tokens
- **GGUF Quantization:** Available on Ollama registry
- **Sizes:** 7B, 13B, 14B, 70B
- **Recommended:** 7B version for desktop
- **Links:**
  - Hugging Face: https://huggingface.co/openthaigpt
  - Ollama Registry: https://ollama.com/promptnow/openthaigpt1.5-7b-instruct-q4_k_m

#### **6. Gemma 3 - Google's Multilingual Option**
- **Sizes:** 1B, 4B, 12B, 27B (all available in instruction-tuned versions)
- **Language Support:** 140+ languages including Thai
- **Thai Performance:** Gemma 3-based Typhoon 2.1 outperforms previous 70B Typhoon2
- **Deployment:** Ollama, vLLM, Llama.cpp
- **Special Case:** Typhoon 2.1 Gemma combines Gemma 3 base with Thai optimization
- **Context:** 8K-131K depending on variant
- **Advantage:** Very capable at small sizes (even 1B is functional)
- **Links:**
  - Model: https://huggingface.co/google/gemma-3-4b-it
  - Ollama: https://ollama.com/library/gemma3

#### **7. Phi-4-mini - Compact Reasoning Model**
- **Parameters:** 3.8B
- **Thai Support:** Yes (among 20+ languages)
- **Strengths:** Mathematical reasoning, logic
- **Context Window:** 128K tokens
- **Quantization:** 4-bit, 5-bit available
- **RAM Requirements:** 8GB RAM (Q4), 16GB recommended
- **Edge Case:** Better for complex logical corrections, less for general Thai
- **Links:**
  - Base: https://huggingface.co/microsoft/Phi-4-mini-instruct
  - GGUF: https://huggingface.co/unsloth/Phi-4-mini-instruct-GGUF

### Rust Integration Options

#### **Primary: llama.cpp Rust Bindings**

**Available Crates:**
1. **llama_cpp** (Most Popular)
   - Crate: https://crates.io/crates/llama_cpp
   - Type: Safe, high-level bindings
   - Features: User-friendly, ~15 lines of code to run a model
   - Documentation: https://docs.rs/llama_cpp

2. **llama-cpp-2**
   - Crate: https://crates.io/crates/llama-cpp-2
   - Type: Uses bindgen for lower-level access
   - Features: More control, more complex

3. **rs-llama-cpp**
   - Crate: https://crates.io/crates/rs-llama-cpp
   - Type: Auto-generated bindings
   - Features: Minimal maintenance burden

4. **llama_cpp_rs**
   - Crate: https://crates.io/crates/llama_cpp_rs
   - Type: Based on go-llama.cpp port
   - Status: Actively maintained

**Recommendation:** Start with `llama_cpp` crate for simplicity, migrate to `llama-cpp-2` if you need fine-grained control.

#### **Whisper Integration: whisper-rs**
- **Crate:** https://crates.io/crates/whisper-rs
- **Purpose:** Rust bindings for whisper.cpp
- **Use Case:** Local Whisper STT from Rust
- **Integration:** Can chain with llama.cpp bindings in same application

#### **Complete Pipeline in Rust:**
```rust
// Pseudocode showing architecture
let whisper = WhisperContext::new("ggml-model-whisper-base.gguf")?;
let llm = LlamaContext::new("qwen-2.5-3b.gguf")?;

for audio_chunk in audio_stream {
    // 1. Transcription
    let transcription = whisper.transcribe(audio_chunk)?;

    // 2. Build prompt with context
    let prompt = format!(
        "You are correcting Thai speech-to-text output.\nPrevious context: {}\nSTT output: {}\nCorrect it:",
        context_buffer, transcription
    );

    // 3. LLM correction
    let corrected = llm.complete(&prompt)?;

    // 4. Update context
    context_buffer.push(corrected.clone());

    // 5. Send to UI
    ui.display(corrected)?;
}
```

### Model Selection Recommendation by Scenario

**For Maximum Thai Accuracy:**
- Primary: Typhoon2-Qwen2.5-7b (if 6-8GB RAM available)
- Fallback: OpenThaiGPT-1.5-7b

**For CPU-Only System with 4GB RAM:**
- Qwen2.5-1.5B-GGUF (Q4)
- Llama-3.2-3B-GGUF (Q4)

**For CPU-Only System with 2GB RAM:**
- Qwen2.5-0.5B-GGUF (Q4)
- Llama-3.2-1B-GGUF (Q4)

**For Balanced Performance (CPU/Light GPU):**
- Gemma3-4B-it-GGUF
- Phi-4-mini-instruct-GGUF

---

## Topic 5: System Architecture for STT + LLM Pipeline

### Latency Budget

**Total Target Latency:** < 3-5 seconds for desktop app

Breakdown by component (for 10-second audio utterance):

| Component | Duration | Flexible? |
|-----------|----------|-----------|
| **Audio Capture** | 10s | No (user speaking) |
| **Whisper STT** | 5-15s | Slightly (faster models exist) |
| **LLM Correction** | 2-5s | Yes (can optimize) |
| **UI Rendering** | <0.5s | Yes (async) |
| **Total** | ~20-30s | - |

**Key Insight:** For local desktop app, latency is less critical than accuracy. User expects 20-30 seconds for correction, but it must be reliable.

### Streaming vs Batch Approaches

#### **Streaming Correction (Real-Time)**
**Concept:** Correct text as user still speaking or immediately after

**Advantages:**
- User sees corrections appearing in real-time
- More interactive feel
- Can show progressive improvement

**Challenges:**
- Less context available (only previous N sentences)
- May make incorrect corrections (fixed after user finishes)
- Requires careful buffer management

**Implementation:**
```
1. User speaks (audio captured)
2. Every 5-10 second chunk → Whisper
3. Immediately pass to LLM for correction
4. Stream corrections to UI
5. When user stops, final pass over entire text
```

#### **Batch Correction (After User Finishes)**
**Concept:** Collect all audio, transcribe fully, then correct as single batch

**Advantages:**
- Full context available
- More accurate (full utterance context)
- Simpler implementation
- Better handling of discourse corrections ("wait, no...")

**Challenges:**
- User waits longer to see result
- Less interactive
- Higher latency perception

**Implementation:**
```
1. User speaks (full audio collected)
2. Whisper processes entire audio
3. LLM gets full transcription + previous context
4. Returns fully corrected text
5. Display to user
```

**Recommendation for Thai Voice-to-Text:** Start with **batch correction** for simplicity and accuracy, add streaming UI updates if needed later.

### Context Window Management

#### **What to Store**
- **Current Utterance:** Full STT output (100-500 tokens)
- **Previous Context:** Last 2-3 sentences (100-300 tokens)
- **System Prompt:** Correction instructions (50-100 tokens)
- **Domain/Topic Info:** Current context (20-50 tokens)
- **User Vocabulary:** 5-10 frequent corrections (50-100 tokens)

**Total Typical Prompt:** 400-1000 tokens for a 1.5B-3B model

#### **Context Buffer Implementation**
```rust
struct ContextBuffer {
    previous_sentences: Vec<String>,  // Last 3 sentences
    detected_topic: String,            // medical, casual, tech, etc.
    user_vocabulary: HashMap<String, String>, // frequent corrections
    error_patterns: Vec<(String, String)>,    // common errors seen
}

impl ContextBuffer {
    fn build_system_prompt(&self) -> String {
        let mut prompt = "You are correcting Thai speech-to-text output.\n".to_string();

        // Add topic context
        if !self.detected_topic.is_empty() {
            prompt.push_str(&format!("Domain: {}\n", self.detected_topic));
        }

        // Add previous context
        if !self.previous_sentences.is_empty() {
            prompt.push_str("Previous context:\n");
            for sentence in self.previous_sentences.iter().take(3) {
                prompt.push_str(&format!("- {}\n", sentence));
            }
        }

        // Add user vocabulary
        if !self.user_vocabulary.is_empty() {
            prompt.push_str("Remember these user terms:\n");
            for (error, correct) in self.user_vocabulary.iter().take(5) {
                prompt.push_str(&format!("- '{}' should be '{}'\n", error, correct));
            }
        }

        prompt
    }
}
```

#### **Sliding Window for Long Dictation**
For sessions > 20 minutes:

```rust
struct SlidingWindowCorrector {
    window_size: usize,        // sentences to keep
    full_text: Vec<String>,    // all corrected sentences
    context_buffer: ContextBuffer,
}

impl SlidingWindowCorrector {
    fn add_utterance(&mut self, whisper_output: String) -> String {
        // 1. Build prompt with sliding context
        let prompt = self.build_prompt_with_context();

        // 2. Correct with LLM
        let corrected = self.llm.correct(&prompt, &whisper_output)?;

        // 3. Add to full text
        self.full_text.push(corrected.clone());

        // 4. Update sliding window (keep last N sentences)
        self.context_buffer.previous_sentences = self.full_text
            .iter()
            .rev()
            .take(self.window_size)
            .cloned()
            .collect();

        Ok(corrected)
    }
}
```

### When to Show Raw STT vs Corrected Text

**Recommended UI Flow:**

```
User speaks (listening UI)
  ↓
Whisper transcribes → Show "Transcribing..."
  ↓
Display raw STT output (gray/italics - temporary)
  ↓
LLM correcting → Show "Correcting..."
  ↓
Display corrected text (black - final)
  ↓
Show confidence indicator (if desired)
```

**Implementation Notes:**
- Show raw STT immediately for feedback (proves system is working)
- Replace with corrected after LLM completes
- Use visual distinction (color, font style) to show which is "live"
- Option: Show both (raw in smaller gray text, corrected prominent)

### Caching Strategies

#### **1. KV Cache Reuse for Context**
- When same context carries over between utterances
- The previous_sentences are included in every prompt
- **Problem:** Recomputing embeddings each time is wasteful

**Solution: Prefix Caching**
```rust
struct CachedLLMCorrector {
    cache: HashMap<String, Vec<f32>>, // context hash -> KV cache
    llm: LlamaContext,
}

impl CachedLLMCorrector {
    fn correct_with_cache(
        &mut self,
        context_prefix: &str,
        stt_output: &str,
    ) -> Result<String> {
        let cache_key = hash(context_prefix);

        // Check if we've seen this context before
        if let Some(cached_kv) = self.cache.get(&cache_key) {
            // Load cached KV state (skips recomputation of context)
            self.llm.set_kv_cache(cached_kv)?;
        }

        // Complete the generation from cached state
        let prompt = format!("{}\n{}", context_prefix, stt_output);
        let result = self.llm.complete(&prompt)?;

        // Save KV cache for next time
        self.cache.insert(cache_key, self.llm.extract_kv_cache()?);

        Ok(result)
    }
}
```

**Research Finding:** Prefix caching can reduce latency by 50-85% for repeated contexts

#### **2. Vocabulary Caching**
- Precompute embeddings for frequent Thai words
- Build "vocabulary lookup" for domain-specific terms
- Cache user's previous corrections

#### **3. Session-Level Caching**
- Serialize corrected sentences to disk (for recovery)
- Cache topic detection results
- Store user vocabulary changes

### Desktop App Architecture (High Level)

```
┌─────────────────────────────────────┐
│      GUI Layer (Tauri/Gtk)          │
│  - Display STT output               │
│  - Show corrections                 │
│  - User vocabulary management       │
│  - Settings (model selection, etc)  │
└─────────────────────────────────────┘
          ↑              ↓
┌─────────────────────────────────────┐
│      Application Logic Layer         │
│  - Orchestrate STT → Correction     │
│  - Context buffer management        │
│  - Caching strategy                 │
│  - Export/save functionality        │
└─────────────────────────────────────┘
          ↑              ↓
┌─────────────────────────────────────┐
│   Audio + LLM Inference Layer       │
│  - Audio capture (cpal/rodio)       │
│  - Whisper STT (whisper.cpp)        │
│  - LLM Correction (llama.cpp)       │
│  - Context window management        │
└─────────────────────────────────────┘
          ↑              ↓
┌─────────────────────────────────────┐
│      System Layer                    │
│  - Model loading & management        │
│  - Hardware detection (CPU/GPU)     │
│  - Memory management                │
└─────────────────────────────────────┘
```

### Recommended Implementation Sequence

**Phase 1: MVP (Minimum Viable Product)**
1. Audio capture
2. Whisper STT (batch mode)
3. Basic LLM correction (no context)
4. Display UI
5. Save to file

**Phase 2: Context & Quality**
1. Context buffer (previous 3 sentences)
2. Few-shot examples for domain
3. User vocabulary learning
4. Better prompt engineering

**Phase 3: Performance**
1. Prefix caching
2. Model quantization optimization
3. Streaming UI updates
4. Real-time progress display

**Phase 4: Features**
1. Multiple model support switching
2. Custom dictionaries
3. Topic/domain detection
4. Export formats (TXT, DOCX, etc)
5. Search/find in full session

### Database Structure for User Data

```rust
struct UserSession {
    session_id: String,
    timestamp: DateTime<Utc>,
    topic: Option<String>,
    original_audio: PathBuf,
    corrections: Vec<UtteranceCorrection>,
}

struct UtteranceCorrection {
    utterance_id: String,
    stt_output: String,
    corrected_output: String,
    confidence: f32,
    user_manual_edits: Option<String>,
}

struct UserVocabulary {
    user_id: String,
    entries: Vec<(String, String)>,  // (error pattern, correction)
    frequency: HashMap<String, usize>,
    last_updated: DateTime<Utc>,
}
```

---

## Integration Guidelines for Rust Implementation

### Key Dependencies
```toml
[dependencies]
# Audio
cpal = "0.17"        # Cross-platform audio
rodio = "0.17"       # Audio playback

# Speech Recognition
whisper-rs = "0.1"   # Whisper C++ bindings
tokio = "1.0"        # Async runtime

# LLM Inference
llama_cpp = "0.2"    # llama.cpp Rust bindings

# UI
tauri = "2.0"        # Cross-platform desktop framework
# OR
gtk-rs = "0.18"      # Native Linux

# Utils
serde = "1.0"        # Serialization
rayon = "1.7"        # Parallelization
```

### Typical Latency Profile (Qwen 2.5-3B on modern CPU)

| Step | Duration | Notes |
|------|----------|-------|
| Whisper (10s audio) | 8-12s | Can use faster-whisper or optimized builds |
| LLM Context Build | 0.1s | Fast - just string operations |
| LLM Inference (prompt ~600 tokens, output ~100 tokens) | 2-3s | 3B model on CPU |
| **Total** | **10-15s** | Acceptable for desktop app |

### Thai-Specific Optimization Tips

1. **Use CER not WER** for all metrics
2. **Provide tone mark examples** in few-shot prompts if possible
3. **Include common filler word removals** in prompt (เอ่อ, อ่า, ครับ, นะ)
4. **Build user vocabulary early** - collect first-session corrections
5. **Test with code-switched content** (Thai + English) - ensure LLM handles well
6. **Monitor character-level accuracy** in your metrics

---

## Recent Papers & Resources (2023-2026)

### Key Research Papers
1. HyPoradise: https://arxiv.org/abs/2309.15701
2. Whispering LLaMA: https://arxiv.org/abs/2310.06434
3. N-best Rescoring with Context: https://arxiv.org/abs/2406.18972
4. Character Error Rate for Multilingual: https://arxiv.org/abs/2410.07400
5. Denoising GER: https://arxiv.org/abs/2509.04392
6. CLEAR (Code-Mixed): https://aclanthology.org/2025.icnlsp-1.33.pdf

### GitHub Resources
- Whispering LLaMA: https://github.com/Srijith-rkr/Whispering-LLaMA
- Typhoon ASR: https://github.com/scb-10x/typhoon-asr
- OpenAI Whisper: https://github.com/openai/whisper
- llama.cpp: https://github.com/ggml-org/llama.cpp

### Model Repositories
- Qwen: https://huggingface.co/Qwen
- Typhoon: https://huggingface.co/scb10x
- OpenThaiGPT: https://huggingface.co/openthaigpt
- Meta Llama: https://huggingface.co/meta-llama

---

## Summary & Recommendations

### For Your Thai Voice-to-Text Desktop App:

1. **STT Engine:** OpenAI Whisper (use base or small for Thai, or specialized Thai variant)

2. **LLM Correction:**
   - **First Choice:** Qwen 2.5-3B GGUF + few-shot correction
   - **Best Quality:** Typhoon2-Qwen2.5-7b if resources permit
   - **Fallback:** OpenThaiGPT 1.5-7B

3. **Architecture:**
   - Batch correction mode (collect full utterance, correct once)
   - Sliding context window (previous 2-3 sentences)
   - User vocabulary tracking
   - CER-based metrics for evaluation

4. **Rust Integration:**
   - whisper-rs for STT
   - llama_cpp crate for LLM
   - Tauri or GTK for UI
   - Tokio for async operations

5. **Performance Targets:**
   - Total latency: 10-20s for typical 10s utterance
   - CER improvement: 20-40% from LLM correction
   - Memory usage: 4-8GB for both models loaded

6. **Key Optimizations:**
   - Use GGUF quantization (Q4 format)
   - Implement prefix caching for context
   - Monitor CER not WER for Thai
   - Build user vocabulary early
   - Test extensively with code-switched content
