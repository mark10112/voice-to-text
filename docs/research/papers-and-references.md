# Research Papers & Technical References
## Thai Voice-to-Text STT + LLM Post-Processing Research

---

## Key Research Papers (2023-2026)

### 1. HyPoradise Dataset - The Current Benchmark

**Title:** HyPoradise: An Open Baseline for Generative Speech Recognition with Large Language Models

**Authors:** Google Research, Published at NeurIPS 2023

**ArXiv:** https://arxiv.org/abs/2309.15701

**Paper URL:** https://papers.neurips.cc/paper_files/paper/2023/file/6492267465a7ac507be1f9fd1174e78d-Paper-Datasets_and_Benchmarks.pdf

**Key Contributions:**
- First open-source benchmark for LLM-based ASR error correction
- Dataset: 316,000+ pairs of N-best hypotheses and accurate transcriptions
- Covers multiple speech domains
- Shows generative correction beats traditional N-best rescoring
- LLM can fix tokens missing from N-best list (major innovation)

**Relevance to Thai Application:**
- Foundation benchmark for measuring correction performance
- Demonstrates that LLM generation (not just ranking) is superior
- Provides evaluation methodology applicable to Thai

**Implementation Notes:**
- Use as comparison point for Thai dataset development
- Evaluate your Thai corrections using HyPoradise methodology
- Adapt few-shot examples based on their approach

---

### 2. Whispering LLaMA

**Title:** Whispering LLaMA: A Cross-Modal Generative Error Correction Framework for Speech Recognition

**Authors:** Srijith Radhakrishnan, NVIDIA Taiwan Research

**Venue:** EMNLP 2023 (Conference on Empirical Methods in Natural Language Processing)

**ArXiv:** https://arxiv.org/abs/2310.06434

**GitHub:** https://github.com/Srijith-rkr/Whispering-LLaMA

**ACL Anthology:** https://aclanthology.org/2023.emnlp-main.618.pdf

**NVIDIA Research:** https://research.nvidia.com/labs/twn/publication/emnlp_2023_whisperingllama/

**Key Contributions:**
- Cross-modal approach combining acoustic + linguistic information
- Uses Whisper encoder + LLaMA decoder
- Parameter-efficient fine-tuning methods
- 37.66% relative WER improvement vs N-best rescoring
- Open-source pre-trained models released

**Technical Details:**
- Encoder: Whisper encoder (frozen) for acoustic embeddings
- Decoder: LLaMA language model for generation
- Input: Audio features from Whisper + N-best hypotheses
- Output: Corrected transcription

**Relevance to Thai Application:**
- Demonstrates fusion of acoustic + linguistic information works
- Shows parameter-efficient adaptation approach
- Could adapt Whisper encoder with Thai-specialized LLaMA variant

**Implementation Considerations:**
- More complex than standard LLM correction
- Requires Whisper encoder embedding extraction
- Better quality but higher computational cost
- Consider for high-quality Thai correction use case

---

### 3. Applying LLMs for Rescoring N-best ASR Hypotheses of Casual Conversations

**Title:** Applying LLMs for Rescoring N-best ASR Hypotheses of Casual Conversations: Effects of Domain Adaptation and Context Carry-over

**Published:** 2024

**ArXiv:** https://arxiv.org/abs/2406.18972

**PromptLayer Discussion:** https://www.promptlayer.com/research-papers/applying-llms-for-rescoring-n-best-asr-hypotheses-of-casual-conversations-effects-of-domain-adaptation-and-context-carry-over

**Key Contributions:**
- Shows context carry-over (previous utterances) significantly improves rescoring
- Tests N-best list rescoring with LLMs
- Analyzes impact of context length on computational cost
- Demonstrates domain adaptation effectiveness

**Important Findings:**
- Shorter context lengths (3-5 sentences) achieve near-optimal performance
- Context carry-over reduces WER by 15-30%
- Smaller context = significantly lower computational cost
- Casual conversations benefit more from context than scripted speech

**Relevance to Thai Application:**
- DIRECTLY APPLICABLE: Shows sliding window context works
- Validates keeping 2-3 previous sentences as context
- Proves shorter context = better latency without much quality loss
- Demonstrates effectiveness on conversational Thai

**Practical Implementation:**
```python
# From the paper's approach:
N-best hypotheses: [hyp1, hyp2, hyp3, hyp4, hyp5]
Previous context: [sent1, sent2, sent3]  # Last 3 sentences

Prompt: "Given context [...], choose the best hypothesis"
LLM rescores all 5, returns best
```

---

### 4. CLEAR: Code-Mixed ASR with LLM-Driven Rescoring

**Title:** CLEAR: Code-Mixed ASR with LLM-Driven Rescoring

**Authors:** Shivam Kumar, Md. Shad Akhtar

**Published:** 2025 (ICNLSP)

**Link:** https://aclanthology.org/2025.icnlsp-1.33.pdf

**Key Contributions:**
- Specifically addresses code-mixed speech (two languages mixed)
- Highly relevant for Thai-English mixed content
- Uses LLM rescoring for code-switched ASR
- Tests on multiple language pairs

**Why Relevant to Thai:**
- Thai-English code-switching is extremely common in Thailand
- Users naturally mix Thai and English in dictation
- Standard models struggle with code-switching
- This paper shows LLM rescoring helps significantly

**Key Insight:**
Code-switching creates homophones and ambiguities that pure LLMs can resolve better than acoustic models alone.

---

### 5. Large Language Model Based Generative Error Correction

**Title:** Large Language Model Based Generative Error Correction: A Challenge and Baselines for Speech Recognition, Speaker Tagging, and Emotion Recognition

**Published:** 2024

**ArXiv:** https://arxiv.org/abs/2409.09785

**Link:** https://arxiv.org/html/2409.09785v3

**Key Contributions:**
- Introduces GenSEC challenge for generative error correction
- Tests on speech recognition, speaker tagging, emotion recognition
- Evaluates multiple LLM approaches
- Shows zero-shot to few-shot improvements

**Multi-Task Aspect:**
- Not just ASR correction, but related tasks
- Demonstrates LLM effectiveness across speech-related tasks
- Could extend Thai application to emotion detection, etc.

---

### 6. Denoising GER: Noise-Robust Generative Error Correction

**Title:** Denoising GER: A Noise-Robust Generative Error Correction with LLM for Speech Recognition

**Authors:** Yanyan Liu, Minqiang Xu, Yihao Chen et al.

**Published:** 2024

**ArXiv:** https://arxiv.org/abs/2509.04392

**Link:** https://arxiv.org/html/2509.04392v1

**Key Contributions:**
- Addresses LLM challenges in noisy environments
- Proposes techniques for poor adaptability in complex conditions
- Improves information utilization in GER

**Relevance:**
- Desktop app STT often deals with real-world noise
- Techniques applicable to Thai in noisy environments
- Shows how to handle low-confidence hypotheses

---

### 7. Character Error Rate for Multilingual ASR Evaluation

**Title:** Advocating Character Error Rate for Multilingual ASR Evaluation

**Authors:** Thennal D K et al.

**Published:** 2025, NAACL Findings

**ArXiv:** https://arxiv.org/abs/2410.07400

**ACL Anthology:** https://aclanthology.org/2025.findings-naacl.277/

**CRITICAL FOR THAI:** This is a must-read for Thai evaluation methodology

**Key Findings:**
- WER fundamentally breaks for languages without word boundaries (Thai, Chinese)
- CER is more consistent and reliable for Thai
- WER can yield 100% error rates despite good transcription
- CER avoids morphological complexity issues

**Key Quote from Paper:**
"WER limitations stem from its sensitivity to morphological complexity and the absence of clear word boundaries in many languages, such as Thai and Chinese, where it can yield 100% error rates."

**Implication for Thai Voice-to-Text:**
- MUST use CER as primary metric, not WER
- Don't report WER for Thai - use CER only
- Comparison with other Thai systems must use CER

**Implementation:**
- Build evaluation script that computes CER
- Track CER improvement (STT-only vs STT+LLM correction)
- Report as: "CER improved from X% to Y% (Z% relative improvement)"

---

### 8. FLANEC: Exploring Flan-T5 for Post-ASR Error Correction

**Title:** FLANEC: Exploring Flan-T5 for Post-ASR Error Correction

**Published:** 2024

**ArXiv:** https://arxiv.org/abs/2501.12979

**Key Insight:**
- Shows that smaller encoder-decoder models work well for ASR correction
- Flan-T5-base and -large comparable to larger models
- Relevant for resource-constrained local deployment

**Implications:**
- Don't need 7B models necessarily
- Smaller models (250M-770M) can be effective
- Consider Flan-T5-base as alternative lightweight option

---

### 9. Using Tone Information in Thai Spelling Speech Recognition

**Title:** Using Tone Information in Thai Spelling Speech Recognition

**Published:** 2014 (ACL)

**Link:** https://aclanthology.org/Y14-1023.pdf

**Older paper but ESSENTIAL for Thai-specific challenges:**

**Key Contributions:**
- Specifically addresses Thai tone marks in ASR
- Shows importance of tone information
- Proposes tone-aware approaches

**Thai-Specific Insights:**
- 5 tones in Thai create homophones
- Tone marks critical for meaning
- Tone information helps resolve ambiguities
- Traditional ASR focuses on phonetics, ignores tones

**For Your Application:**
- Include tone mark examples in few-shot prompts
- Note that correcting tone marks = correcting meaning
- Example: `ม้า` (horse) vs `มา` (come) - same sound, different tone marks

---

### 10. Homophone Identification and Merging for Code-switched Speech Recognition

**Title:** Homophone Identification and Merging for Code-switched Speech Recognition

**Authors:** Srivastava et al.

**Venue:** Interspeech 2018

**ResearchGate:** https://www.researchgate.net/publication/327388676_Homophone_Identification_and_Merging_for_Code-switched_Speech_Recognition

**ISCA Archive:** https://www.isca-archive.org/interspeech_2018/srivastava18_interspeech.html

**Key Contribution:**
- Addresses homophones when code-switching (Thai + English)
- 1.2% relative WER improvement with homophone merging
- Shows standardization strategy for code-switched transcription

**Application to Thai:**
- Thai naturally mixes with English
- This technique helps disambiguate mixed-language homophones
- Useful for building better correction prompts

---

## Quick Reference: Metric Definitions

### Character Error Rate (CER)

**Definition:**
```
CER = (S + D + I) / N

Where:
  S = number of character substitutions
  D = number of character deletions
  I = number of character insertions
  N = total number of characters in reference
```

**Example (Thai):**
```
Reference:  สวัสดีครับ (8 characters)
Hypothesis: สวัส ดี ครับ (with spaces, effectively wrong segmentation)

After removing spaces for fair comparison:
Reference:  สวัสดีครับ
Hypothesis: สวัสดีครับ (same)
CER = 0%

If hypothesis was "สวัส ดิ ครับ":
Reference:  สวัสดีครับ
Hypothesis: สวัสดิครับ
One character mismatch: ี→ิ
CER = 1/8 = 12.5%
```

**Why CER for Thai:**
- Thai has no word boundaries
- Segmentation differs between systems
- CER ignores spaces/boundaries
- More robust across systems

### Word Error Rate (WER)

**Definition:**
```
WER = (S + D + I) / N

Where:
  S = number of word substitutions
  D = number of word deletions
  I = number of word insertions
  N = total number of words in reference
```

**Problem for Thai:**
- Thai word boundaries are ambiguous
- Same text can be segmented 5+ different ways
- WER highly sensitive to segmentation choice
- Can give 100% WER on actually correct output

**Example (Thai):**
```
Correct text could be segmented as:
1. สวัส/ดี/ครับ (3 words)
2. สวัสดี/ครับ (2 words)
3. สวัส/ดี ครับ (2 words)

Different segmentations = different WER even for same transcription!
```

---

## Model Comparison Quick Reference

### Performance Tiers by Parameter Count

```
Lightweight (For CPU-Only Desktop):
- 0.5B: Qwen 2.5-0.5B (~300MB GGUF Q4)
- 1B:   Llama 3.2-1B, Qwen 2.5-1.5B (~800MB Q4)
- 3B:   Llama 3.2-3B, Qwen 2.5-3B (~2GB Q4)

Standard (Laptop/Low-End Desktop):
- 7B:   Typhoon2-Qwen2.5, OpenThaiGPT-7B, Phi-4-mini (3.8B)

High Quality (High-End Desktop):
- 13-14B: OpenThaiGPT-13B/14B
- 70B:    Typhoon, OpenThaiGPT (requires GPU)
```

### Thai Language Support Matrix

| Model | Thai | Code-Switch | Context | GGUF | CPU |
|-------|------|-------------|---------|------|-----|
| Qwen 2.5 | ✓ | ✓ | 32K | ✓ | ✓ |
| Typhoon2-Qwen | ✓✓ | ✓✓ | 128K | ✓ | ✓(slow) |
| Llama 3.2 | ✓ | ✓ | 8K | ✓ | ✓ |
| Typhoon2 | ✓✓✓ | ✓ | 8K | ✓ | ✗ |
| OpenThaiGPT | ✓✓✓ | ✓ | 131K | ✓ | ✗ |
| Gemma3 | ✓ | ✓ | 8K+ | ✓ | ✓ |
| Phi-4-mini | ✓ | ✓ | 128K | ✓ | ✓ |

---

## GitHub Repositories & Open Source Projects

### Core Projects

#### 1. OpenAI Whisper
- **URL:** https://github.com/openai/whisper
- **Language:** Python
- **Purpose:** Official Whisper STT implementation
- **Key Features:**
  - Multilingual (supports Thai)
  - Open weights
  - CPU-capable
- **For Rust:** Use whisper.cpp or whisper-rs bindings

#### 2. Whispering LLaMA
- **URL:** https://github.com/Srijith-rkr/Whispering-LLaMA
- **Language:** Python
- **Purpose:** Cross-modal error correction
- **Key Features:**
  - Whisper encoder + LLaMA decoder
  - Pre-trained models available
  - EMNLP 2023 publication

#### 3. Typhoon (SCB10X)
- **Main Site:** https://opentyphoon.ai/
- **Models:** https://huggingface.co/scb10x
- **GitHub:** https://github.com/scb-10x/typhoon*
- **Language:** Multiple repos
- **Subprojects:**
  - **Typhoon ASR:** https://github.com/scb-10x/typhoon-asr (Real-time streaming ASR for Thai)
  - **Typhoon Audio:** https://github.com/scb-10x/typhoon2-audio (Speech in/out)
  - **Typhoon LLM:** Main language model

#### 4. OpenThaiGPT
- **GitHub:** https://github.com/OpenThaiGPT
- **Hugging Face:** https://huggingface.co/openthaigpt
- **Language:** Python/Multiple
- **Purpose:** Thai-focused LLM
- **Key Models:**
  - openthaigpt-1.0.0-7b-chat
  - openthaigpt-1.5-7b-instruct (newer, Qwen-based)
  - Available in GGUF format

#### 5. llama.cpp
- **URL:** https://github.com/ggml-org/llama.cpp
- **Language:** C++
- **Purpose:** Efficient LLM inference
- **Key Features:**
  - GGUF format support
  - CPU optimized
  - Multiple language bindings
- **Rust Bindings:** llama_cpp crate

#### 6. faster-whisper
- **URL:** https://github.com/SYSTRAN/faster-whisper
- **Language:** Python
- **Purpose:** Optimized Whisper (4x faster)
- **Key Features:**
  - CTranslate2 optimization
  - Still supports Thai
  - Lower latency

### Supporting Tools

#### Audio Processing
- **hound:** Rust WAV file reading
- **cpal:** Cross-platform audio capture
- **rodio:** Audio playback

#### LLM Tools
- **ollama:** Simple local LLM runner
- **vLLM:** Batch inference optimization
- **llm-studio:** Model fine-tuning UI

---

## Hugging Face Model Collections

### Thai Language Models

**Direct Search:** https://huggingface.co/models?language=th

**Key Collections:**
- Typhoon: https://huggingface.co/scb10x
- OpenThaiGPT: https://huggingface.co/openthaigpt
- Qwen: https://huggingface.co/Qwen

### GGUF Quantized Versions

**Qwen2.5:**
- Bartowski quantizations: https://huggingface.co/bartowski
- Unsloth quantizations: https://huggingface.co/unsloth

**Llama 3.2:**
- Bartowski: https://huggingface.co/bartowski
- Unsloth: https://huggingface.co/unsloth
- QuantFactory: https://huggingface.co/QuantFactory

**Phi-4-mini:**
- Unsloth: https://huggingface.co/unsloth
- Bartowski: https://huggingface.co/bartowski

---

## Important Benchmarks & Datasets

### Whisper Evaluation

**Common Voice 15 Dataset:** https://commonvoice.mozilla.org/
- Large multilingual dataset
- Includes Thai
- Published baselines available

**FLEURS Dataset:**
- Multilingual evaluation
- Thai included
- Standardized evaluation set

**Whisper Paper Appendix:**
- Contains WER/CER metrics for all supported languages
- Reference: "Robust Speech Recognition via Large-Scale Weak Supervision"

### HyPoradise Benchmark
- **Dataset:** https://github.com/lizhenmei/HyPoradise
- **Size:** 316K+ examples
- **Purpose:** ASR error correction benchmark
- Consider creating Thai equivalent

---

## Key Metrics to Track

### For Your Thai Application

1. **CER (Character Error Rate)** - Primary metric
   - Before correction: STT-only
   - After correction: STT + LLM
   - Target: 30-50% relative improvement

2. **Inference Latency**
   - STT time (5-15s for typical 10s audio)
   - LLM correction time (1-3s for typical sentence)
   - Total: aim for < 20s per utterance

3. **Memory Usage**
   - Model size (GGUF Q4: 2-8GB depending on size)
   - Runtime memory (buffer + state): 100-500MB
   - Total: should fit in 8GB system

4. **Domain-Specific Accuracy**
   - Track CER by topic (medical, casual, tech)
   - Measure vocabulary learning efficiency
   - Monitor user satisfaction

---

## Paper Reading Priority

**Priority 1 (Must Read):**
1. HyPoradise (understand evaluation methodology)
2. CER for Multilingual ASR (methodology for Thai evaluation)
3. Applying LLMs for Rescoring (context strategy validation)

**Priority 2 (Important for Thai):**
1. Using Tone Information in Thai (tone-specific challenges)
2. Homophone Identification (code-switching handling)
3. Whispering LLaMA (architecture reference)

**Priority 3 (Nice to Have):**
1. CLEAR (code-mixing specific)
2. FLANEC (lightweight model options)
3. Denoising GER (noise robustness)

---

## Key Takeaways from Literature

1. **LLM Generative Correction > N-best Rescoring**
   - Generate new text rather than select from candidates
   - Can fix words outside N-best list
   - Better quality but slightly more latency

2. **Context is Critical**
   - Previous 2-3 sentences sufficient for good results
   - Longer context has diminishing returns
   - Sliding window approach proven effective

3. **Thai Requires CER Evaluation**
   - WER is unreliable for Thai
   - Must use CER as primary metric
   - Document that CER improvement of 30%+ is excellent

4. **Small Context, Fast Inference**
   - 3-5B models on CPU are practical
   - GGUF Q4 quantization works well
   - Smaller context windows (1K-2K tokens) sufficient

5. **Few-Shot Examples Help**
   - 2-5 examples of correct formatting
   - Domain-specific examples boost accuracy
   - User vocabulary learning essential for personalization

6. **Code-Switching is Hard**
   - Thai-English mixing creates unique challenges
   - Standard approaches struggle
   - LLM-based correction helps significantly

---

## Citation Style References

If you build on this research, cite these papers:

```
@inproceedings{hyparadise2023,
  title={HyPoradise: An Open Baseline for Generative Speech Recognition with Large Language Models},
  booktitle={NeurIPS},
  year={2023}
}

@inproceedings{whisperingllama2023,
  title={Whispering LLaMA: A Cross-Modal Generative Error Correction Framework for Speech Recognition},
  author={Radhakrishnan, Srijith and others},
  booktitle={EMNLP},
  year={2023}
}

@article{cer_multilingual2025,
  title={Advocating Character Error Rate for Multilingual ASR Evaluation},
  author={Thennal DK and others},
  journal={NAACL Findings},
  year={2025}
}

@article{nbest_context2024,
  title={Applying LLMs for Rescoring N-best ASR Hypotheses of Casual Conversations: Effects of Domain Adaptation and Context Carry-over},
  year={2024},
  journal={arXiv}
}
```

---

## Additional Resources

### Thai NLP Resources
- PyThaiNLP: https://github.com/PyThaiNLP/pythainlp
- Thai Language Tools: https://github.com/PyThaiNLP

### Model Hubs
- Hugging Face: https://huggingface.co/
- Ollama: https://ollama.com/ (easy local deployment)
- Replicate: https://replicate.com/ (API option)

### Conferences to Follow
- Interspeech (speech recognition)
- ICNLSP (natural language processing)
- NAACL (NLP conference)
- EMNLP (empirical methods)

---

**Last Updated:** February 2026
**Status:** Comprehensive research compilation for Thai voice-to-text application
