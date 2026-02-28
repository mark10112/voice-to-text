# Comparison: Thai STT Engines & Competitor Apps

**วันที่:** 28 กุมภาพันธ์ 2026

---

## Part 1: เปรียบเทียบ Thai STT Engines

### Benchmark: FLEURS Thai Dataset (ค่ายิ่งต่ำยิ่งดี)

| Engine | Thai WER/CER | Latency | Online/Offline | Cost | Open Source |
|--------|-------------|---------|---------------|------|-------------|
| **ElevenLabs Scribe** | **3.1% WER** ⭐ | Real-time | Cloud only | API (paid) | ❌ |
| **Speechmatics** | ~0.5–2.5% WER (real-world) | <1s | Cloud only | Enterprise | ❌ |
| **Thonburian Whisper** | ดีกว่า Whisper base | ~5–15s | ✅ Offline | **Free** | ✅ |
| **OpenAI Whisper Large-V3** | สูงกว่า ElevenLabs | ~9–10s | Both | Free / $0.02/min | ✅ |
| **Whisper Large-V3-Turbo** | สูงกว่า v3 (Thai drops more) | 6x เร็ว | Both | Free / $0.02/min | ✅ |
| **Faster-Whisper** | เท่ากับ base | 4x เร็วกว่า base | ✅ Offline | **Free** | ✅ |
| **AssemblyAI Universal-2** | ไม่มีข้อมูล Thai | Streaming | Cloud only | API (paid) | ❌ |
| **Deepgram Nova-3** | ไม่มีข้อมูล Thai | <300ms | Cloud only | $0.0077/min | ❌ |
| **Google Chirp 3** | ไม่เปิดเผย | ~3–5s | Cloud only | $0.024/15s | ❌ |
| **Azure Speech** | ไม่มีข้อมูล Thai | ~2–3s | Cloud only | Pay-per-use | ❌ |
| **Vosk** | ❌ ไม่รองรับ Thai | เร็ว | ✅ Offline | Free | ✅ |

### วิเคราะห์ตาม Use Case

**ถ้าต้องการ accuracy สูงสุด (ยอมจ่ายเงิน):**
→ ElevenLabs Scribe หรือ Speechmatics

**ถ้าต้องการ offline + ฟรี + Thai specific:**
→ Thonburian Whisper (แนะนำ) หรือ Faster-Whisper

**ถ้าต้องการ speed สูงสุด (ยอมใช้ cloud):**
→ Deepgram Nova-3 (<300ms) หรือ Speechmatics (<1s)

**ถ้าต้องการ open source ทั้งหมด:**
→ Thonburian Whisper + whisper-rs + Faster-Whisper

---

## Part 2: เปรียบเทียบ Competitor Apps

### Voice-to-Text Widget Apps

| App | Platform | Thai | System-wide | Cost | Offline | Open Source |
|-----|----------|------|-------------|------|---------|-------------|
| **Windows Voice Typing** (Win+H) | Windows เท่านั้น | ✅ ใช่ | ✅ | ฟรี | ❌ Cloud | ❌ |
| **Wispr Flow** | Win / Mac / iOS | ✅ ใช่ | ✅ | ฟรี + $12/เดือน | ❓ Hybrid | ❌ |
| **Superwhisper** | **macOS เท่านั้น** | ❓ น่าจะได้ | ✅ | $4.99/เดือน | ✅ | ❌ |
| **VoiceIn** | Browser Extension | ✅ ใช่ | ❌ Browser เท่านั้น | ฟรี + paid | ✅ Browser | ❌ |
| **WhisperTyping** | Windows เท่านั้น | ❌ ไม่รองรับ | ✅ | ฟรี + paid | ❌ Cloud | ❌ |
| **Talon Voice** | Win/Mac/Linux | ❌ ไม่รองรับ | ✅ | ฟรี | ✅ | บางส่วน |
| **Dragon Professional** | Windows (หลัก) | ❌ ไม่รองรับ | ✅ | $150 | ✅ | ❌ |
| **Google Docs Voice** | Browser (Chrome) | ❓ น่าจะได้ | ❌ Docs เท่านั้น | ฟรี | ❌ Cloud | ❌ |
| **🎯 โปรเจคของเรา** | **Win/Mac/Linux** | ✅ **เต็ม** | ✅ | **ฟรี (open source)** | ✅ | ✅ |

### วิเคราะห์ช่องว่างในตลาด (Gap Analysis)

```
ปัญหาของ competitors:
┌────────────────────────────────────────────────────┐
│ Windows Voice Typing  → Windows เท่านั้น, cloud    │
│ Superwhisper          → macOS เท่านั้น             │
│ Wispr Flow            → ไม่มี Linux, paid          │
│ WhisperTyping         → ไม่รองรับ Thai, Windows เท่านั้น │
│ VoiceIn               → Browser เท่านั้น           │
│ Talon/Dragon          → ไม่รองรับ Thai             │
└────────────────────────────────────────────────────┘

✅ โอกาส:
  ไม่มี app ไหนที่ครบทั้ง:
  1. รองรับภาษาไทย
  2. ทำงาน cross-platform (Win+Mac+Linux)
  3. ทำงาน offline / privacy-first
  4. System-wide (ใช้ได้ทุก app)
  5. Open source + ฟรี
```

---

## Part 3: สรุปข้อเสนอแนะสำหรับโปรเจค

### Model ที่ควรใช้ (เรียงลำดับ)

**Primary:**
- **Thonburian Whisper Medium** — ดีที่สุดสำหรับ Thai offline
  - `biodatlab/whisper-th-medium-combined`
  - Balance ระหว่าง accuracy vs speed

**Fallback / Advanced:**
- **Thonburian Whisper Large** สำหรับผู้ใช้ที่มี GPU
- **Faster-Whisper + Thonburian weights** ถ้าต้องการ speed
  - ใช้ CTranslate2 format

**Cloud Mode (optional):**
- **ElevenLabs Scribe API** — ถ้า user ยอม internet + เสียค่าใช้จ่าย
  - เพิ่มเป็น optional feature ในอนาคต

### Architecture ที่แนะนำเพิ่มเติม

```
User เลือก mode:
├── Local Mode (default): Thonburian Whisper Medium
│   └── ไม่ต้องใช้ internet, privacy 100%
└── Cloud Mode (optional): ElevenLabs Scribe
    └── accuracy ดีกว่า, ต้องใช้ internet
```

---

## Sources

- [ElevenLabs Scribe Blog](https://elevenlabs.io/blog/meet-scribe)
- [Soniox STT Benchmarks 2025](https://soniox.com/benchmarks)
- [Thonburian Whisper Paper ICNLSP 2024](https://aclanthology.org/2024.icnlsp-1.17.pdf)
- [Speechmatics Thai STT](https://www.speechmatics.com/speech-to-text/thai)
- [Faster-Whisper GitHub](https://github.com/SYSTRAN/faster-whisper)
- [Wispr Flow Pricing](https://wisprflow.ai/pricing)
- [VoiceIn Thai Support](https://dictanote.co/voicein/languages/thai/)
- [Superwhisper Website](https://superwhisper.com/)
- [Windows Voice Typing](https://support.microsoft.com/en-us/windows/use-voice-typing-to-talk-instead-of-type-on-your-pc-fec94565-c4bd-329d-e59a-af033fa5689f)
- [Dragon Language Support](https://nuance.custhelp.com/app/answers/detail/a_id/3315)
