# Threading & Data Flow Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** Thread architecture, channels, state synchronization, error propagation

---

## 1. Thread Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Process: thai-stt                       │
│                                                           │
│  ┌─────────────────┐                                     │
│  │  Main Thread     │  egui UI loop + event handling     │
│  │  (eframe)        │  ← runs at vsync (~60fps idle)     │
│  └────────┬─────────┘                                    │
│           │                                              │
│           │  mpsc channels                               │
│           │                                              │
│  ┌────────▼─────────┐                                    │
│  │  Hotkey Thread    │  rdev::listen() — blocking loop   │
│  │  (dedicated)      │  Sends: HotkeyEvent              │
│  └──────────────────┘                                    │
│                                                           │
│  ┌──────────────────┐                                    │
│  │  Audio Thread     │  cpal stream callback             │
│  │  (cpal-managed)   │  Writes: samples → AudioBuffer    │
│  └──────────────────┘                                    │
│                                                           │
│  ┌──────────────────┐                                    │
│  │  STT Thread       │  whisper-rs inference (blocking)  │
│  │  (tokio spawn_    │  Reads: AudioBuffer               │
│  │   blocking)       │  Sends: TranscriptionResult       │
│  └──────────────────┘                                    │
│                                                           │
│  ┌──────────────────┐                                    │
│  │  LLM Thread       │  Ollama HTTP call (async)         │
│  │  (tokio task)     │  Sends: CorrectionResult          │
│  └──────────────────┘                                    │
│                                                           │
│  ┌──────────────────┐                                    │
│  │  Inject Thread    │  Clipboard + key simulation       │
│  │  (tokio spawn_    │  (blocking, brief)                │
│  │   blocking)       │                                   │
│  └──────────────────┘                                    │
└──────────────────────────────────────────────────────────┘
```

---

## 2. Channel Design

### 2.1 Channel Map

```
Hotkey Thread ───── HotkeyEvent ────────────▶ Main Thread (UI)
                                                    │
                                         ┌──────────┼────────────┐
                                         │          │            │
                                         ▼          ▼            ▼
                                    Start/Stop   Pipeline     Update
                                    Recording    Commands     UI State
                                         │
                                         ▼
Audio Thread ─── samples ──▶ AudioBuffer (Arc<Mutex<>>)
                                         │
                                    drain buffer
                                         │
                                         ▼
                              STT Thread (spawn_blocking)
                                         │
                                  TranscriptionResult
                                         │
                                         ▼
                              LLM Thread (async task)
                                         │
                                  CorrectionResult
                                         │
                                         ▼
                              Inject Thread (spawn_blocking)
                                         │
                                  InjectResult ──────▶ Main Thread (UI)
```

### 2.2 Message Types

```rust
/// Commands from UI → Pipeline
pub enum PipelineCommand {
    StartRecording,
    StopRecording,
    Cancel,
    ChangeMode(OperatingMode),
}

/// Events from Hotkey → UI
pub enum HotkeyEvent {
    PushToTalkPressed,
    PushToTalkReleased,
    ToggleVisibility,
}

/// Results from Pipeline → UI
pub enum PipelineResult {
    RecordingStarted,
    RecordingStopped { duration_secs: f32 },
    TranscriptionComplete(TranscriptionResult),
    CorrectionComplete(CorrectionResult),
    InjectionComplete,
    Error(PipelineError),
}

pub struct TranscriptionResult {
    pub raw_text: String,
    pub duration_ms: u128,
}

pub struct CorrectionResult {
    pub corrected_text: String,
    pub duration_ms: u128,
}
```

### 2.3 Channel Setup

```rust
use tokio::sync::mpsc;

pub struct Channels {
    // Hotkey → Main
    pub hotkey_rx: mpsc::Receiver<HotkeyEvent>,
    pub hotkey_tx: mpsc::Sender<HotkeyEvent>,

    // Main → Pipeline
    pub command_tx: mpsc::Sender<PipelineCommand>,
    pub command_rx: mpsc::Receiver<PipelineCommand>,

    // Pipeline → Main
    pub result_tx: mpsc::Sender<PipelineResult>,
    pub result_rx: mpsc::Receiver<PipelineResult>,
}

impl Channels {
    pub fn new() -> Self {
        let (hotkey_tx, hotkey_rx) = mpsc::channel(16);
        let (command_tx, command_rx) = mpsc::channel(16);
        let (result_tx, result_rx) = mpsc::channel(16);

        Self {
            hotkey_tx, hotkey_rx,
            command_tx, command_rx,
            result_tx, result_rx,
        }
    }
}
```

---

## 3. Shared State

### 3.1 Audio Buffer (Hot Path)

Audio callback ทำงานบน real-time thread — ต้อง lock-free หรือ lock สั้นที่สุด:

```rust
use std::sync::{Arc, Mutex};

pub type SharedAudioBuffer = Arc<Mutex<AudioBuffer>>;

// cpal callback → push samples
// Pipeline thread → drain buffer
```

**ทำไมใช้ Mutex ไม่ใช่ lock-free ring buffer:**
- Push-to-talk: lock duration สั้นมาก (microseconds)
- ไม่ใช่ real-time streaming — ไม่ต้อง lock-free
- ง่ายกว่า, ถูกต้องกว่า
- ถ้ามีปัญหา latency → เปลี่ยนเป็น `ringbuf` crate ทีหลัง

### 3.2 App State (UI Thread Only)

```rust
/// State ที่ UI thread เป็นเจ้าของ — ไม่ต้อง share
pub struct AppState {
    pub pipeline_state: PipelineState,
    pub raw_text: Option<String>,
    pub corrected_text: Option<String>,
    pub waveform: Vec<f32>,
    pub settings: AppSettings,
    pub error: Option<String>,
}
```

### 3.3 Settings (Read-heavy)

```rust
use std::sync::Arc;
use parking_lot::RwLock;

/// Settings อ่านบ่อย เขียนน้อย — ใช้ RwLock
pub type SharedSettings = Arc<RwLock<AppSettings>>;
```

---

## 4. Pipeline Orchestrator

### 4.1 Orchestrator Loop

```rust
pub struct PipelineOrchestrator {
    audio_buffer: SharedAudioBuffer,
    stt_engine: WhisperEngine,
    llm_corrector: OllamaCorrector,
    text_injector: TextInjector,
    context_manager: ContextManager,
    settings: SharedSettings,
}

impl PipelineOrchestrator {
    pub async fn run(
        mut self,
        mut command_rx: mpsc::Receiver<PipelineCommand>,
        result_tx: mpsc::Sender<PipelineResult>,
    ) {
        while let Some(cmd) = command_rx.recv().await {
            match cmd {
                PipelineCommand::StartRecording => {
                    self.audio_buffer.lock().unwrap().clear();
                    // cpal stream เริ่ม capture อยู่แล้ว
                    // เปลี่ยน flag ให้ buffer รับ samples
                    self.audio_buffer.lock().unwrap().is_recording = true;
                    let _ = result_tx.send(PipelineResult::RecordingStarted).await;
                }

                PipelineCommand::StopRecording => {
                    // 1. Stop recording
                    let audio = {
                        let mut buf = self.audio_buffer.lock().unwrap();
                        buf.is_recording = false;
                        buf.drain()
                    };

                    let duration = audio.len() as f32 / 16_000.0;
                    let _ = result_tx.send(
                        PipelineResult::RecordingStopped { duration_secs: duration }
                    ).await;

                    // 2. Transcribe (blocking — run on thread pool)
                    let stt = self.stt_engine.clone();
                    let tx = result_tx.clone();
                    let audio_clone = audio.clone();

                    let stt_result = tokio::task::spawn_blocking(move || {
                        stt.transcribe(&audio_clone)
                    }).await;

                    match stt_result {
                        Ok(Ok(result)) => {
                            let raw_text = result.text.clone();
                            let _ = tx.send(
                                PipelineResult::TranscriptionComplete(
                                    TranscriptionResult {
                                        raw_text: raw_text.clone(),
                                        duration_ms: result.duration_ms,
                                    }
                                )
                            ).await;

                            // 3. LLM correction (if enabled)
                            let mode = self.settings.read().operating_mode;
                            if mode != OperatingMode::Fast {
                                let context = self.context_manager.build_context();
                                match self.llm_corrector.correct(&raw_text, &context).await {
                                    Ok(corrected) => {
                                        self.context_manager.push_sentence(corrected.clone());
                                        let _ = tx.send(
                                            PipelineResult::CorrectionComplete(
                                                CorrectionResult {
                                                    corrected_text: corrected.clone(),
                                                    duration_ms: 0, // TODO
                                                }
                                            )
                                        ).await;

                                        // 4. Inject
                                        let injector = self.text_injector.clone();
                                        let _ = tokio::task::spawn_blocking(move || {
                                            injector.inject(&corrected)
                                        }).await;
                                    }
                                    Err(_) => {
                                        // Fallback: inject raw text
                                        let injector = self.text_injector.clone();
                                        let raw = raw_text.clone();
                                        let _ = tokio::task::spawn_blocking(move || {
                                            injector.inject(&raw)
                                        }).await;
                                    }
                                }
                            } else {
                                // Fast mode: inject raw text
                                let injector = self.text_injector.clone();
                                let raw = raw_text.clone();
                                let _ = tokio::task::spawn_blocking(move || {
                                    injector.inject(&raw)
                                }).await;
                            }

                            let _ = tx.send(PipelineResult::InjectionComplete).await;
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(PipelineResult::Error(
                                PipelineError::Stt(e.to_string())
                            )).await;
                        }
                        Err(e) => {
                            let _ = tx.send(PipelineResult::Error(
                                PipelineError::Internal(e.to_string())
                            )).await;
                        }
                    }
                }

                PipelineCommand::Cancel => {
                    self.audio_buffer.lock().unwrap().is_recording = false;
                    self.audio_buffer.lock().unwrap().clear();
                }

                PipelineCommand::ChangeMode(mode) => {
                    self.settings.write().operating_mode = mode;
                }
            }
        }
    }
}
```

---

## 5. Hotkey Thread

### 5.1 rdev Listener

```rust
pub fn spawn_hotkey_listener(tx: mpsc::Sender<HotkeyEvent>, hotkey: Key) {
    std::thread::spawn(move || {
        rdev::listen(move |event| {
            match event.event_type {
                rdev::EventType::KeyPress(key) if key == hotkey => {
                    let _ = tx.blocking_send(HotkeyEvent::PushToTalkPressed);
                }
                rdev::EventType::KeyRelease(key) if key == hotkey => {
                    let _ = tx.blocking_send(HotkeyEvent::PushToTalkReleased);
                }
                _ => {}
            }
        }).expect("Failed to listen for hotkeys");
    });
}
```

### 5.2 ทำไมใช้ dedicated thread

- `rdev::listen()` เป็น blocking call ที่ไม่ return
- ต้องแยก thread — ไม่สามารถใช้ tokio task ได้
- ใช้ `blocking_send()` เพื่อส่ง events กลับ

---

## 6. Startup Sequence

```
main()
  │
  ├── 1. Load config from disk
  ├── 2. Create channels
  ├── 3. Initialize AudioBuffer (shared)
  │
  ├── 4. Spawn tokio runtime (background)
  │     ├── Spawn PipelineOrchestrator task
  │     └── Load Whisper model (blocking)
  │
  ├── 5. Spawn hotkey listener thread
  │
  ├── 6. Start cpal audio stream (paused)
  │
  └── 7. Run eframe::run_native() (blocks main thread)
         └── UI loop polls channels each frame
```

```rust
fn main() -> eframe::Result<()> {
    // 1. Config
    let settings = SharedSettings::new(RwLock::new(AppSettings::load()));

    // 2. Channels
    let channels = Channels::new();

    // 3. Audio buffer
    let audio_buffer = SharedAudioBuffer::new(Mutex::new(AudioBuffer::new(60)));

    // 4. Tokio runtime (for async pipeline)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    // Spawn pipeline
    let orchestrator = PipelineOrchestrator::new(
        audio_buffer.clone(),
        settings.clone(),
    );
    rt.spawn(orchestrator.run(channels.command_rx, channels.result_tx));

    // 5. Hotkey
    spawn_hotkey_listener(channels.hotkey_tx, rdev::Key::F9);

    // 6. cpal stream (started in AudioCapture::new)
    let audio_capture = AudioCapture::new(audio_buffer.clone())?;

    // 7. UI
    let app = ThaiSttApp::new(
        channels.hotkey_rx,
        channels.command_tx,
        channels.result_rx,
        settings,
    );

    eframe::run_native("Thai STT", native_options(), Box::new(|_| Ok(Box::new(app))))
}
```

---

## 7. Error Propagation

```rust
#[derive(Debug)]
pub enum PipelineError {
    Audio(String),
    Stt(String),
    Llm(String),
    Inject(String),
    Internal(String),
}
```

**กลยุทธ์:** ทุก error ส่งกลับ UI ผ่าน `PipelineResult::Error` — UI ตัดสินใจว่าจะ
แสดงอะไร / fallback อย่างไร

---

## 8. Shutdown Sequence

```
User clicks close / Ctrl+C
  │
  ├── 1. Drop cpal stream (stops audio callback)
  ├── 2. Send Cancel to pipeline
  ├── 3. Drop channel senders (causes receivers to close)
  ├── 4. tokio runtime shuts down (waits for tasks)
  ├── 5. Hotkey thread exits (rdev::listen has no clean shutdown)
  └── 6. Process exits
```

> **หมายเหตุ:** `rdev::listen()` ไม่มี graceful shutdown — thread จะถูก kill ตอน process exit ซึ่งปลอดภัยเพราะไม่มี state ที่ต้อง cleanup

---

## 9. Performance Considerations

| Concern | Mitigation |
|---------|-----------|
| Audio callback latency | Mutex lock สั้นมาก, push only |
| Whisper blocking UI | `spawn_blocking` แยก thread pool |
| Ollama network latency | Async + timeout 10s |
| UI responsiveness | Poll channels non-blocking ใน `update()` |
| Memory spikes | AudioBuffer มี capacity limit |

---

## 10. Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "sync", "macros"] }
rdev = "0.5"
parking_lot = "0.12"   # Fast RwLock for settings
crossbeam-channel = "0.5"  # Alternative to tokio::sync::mpsc for std threads
```
