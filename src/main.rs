//! Application entry point — Thai Voice-to-Text.
//!
//! # Startup sequence
//!
//! 1. Initialise logging.
//! 2. Load [`AppConfig`] from disk (returns default on first run).
//! 3. Create [`tokio`] runtime (multi-thread, 2 workers).
//! 4. Build the LLM corrector ([`ApiCorrector`]) from config.
//! 5. Create pipeline channels (`command`, `result`).
//! 6. Spawn the pipeline orchestrator on the tokio runtime.
//! 7. Spawn the hotkey listener thread.
//! 8. Start the cpal audio capture stream.
//! 9. Run [`eframe::run_native`] — blocks the main thread until the window
//!    is closed.

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use voice_to_text::{
    app::{PipelineCommand, PipelineResult, ThaiSttApp},
    audio::{AudioCapture, AudioChunk},
    config::AppConfig,
    hotkey::{HotkeyListener, parse_key},
    inject::TextInjector,
    llm::{ApiCorrector, ContextManager, FallbackCorrector, LlmCorrector},
    stt::{SttEngine, TranscribeParams, WhisperEngine},
};

use eframe::egui;

// ---------------------------------------------------------------------------
// Shared audio accumulation buffer
// ---------------------------------------------------------------------------

/// Thread-shared buffer that accumulates audio chunks while recording is
/// active.  The cpal callback and the pipeline orchestrator both access this
/// via `Arc<Mutex<…>>`.
type SharedAudioBuffer = Arc<Mutex<(Vec<f32>, bool)>>;
//                                  ^^^^^^^  ^^^^
//                               samples   is_recording

// ---------------------------------------------------------------------------
// Pipeline orchestrator
// ---------------------------------------------------------------------------

/// Minimal pipeline orchestrator that runs inside the tokio runtime.
///
/// Listens for [`PipelineCommand`]s, drives STT + LLM, and emits
/// [`PipelineResult`]s back to the UI.
async fn run_pipeline(
    audio_buf: SharedAudioBuffer,
    stt: Arc<dyn SttEngine>,
    llm: Arc<dyn LlmCorrector>,
    injector: TextInjector,
    config: AppConfig,
    mut command_rx: mpsc::Receiver<PipelineCommand>,
    result_tx: mpsc::Sender<PipelineResult>,
) {
    let mut ctx_mgr = ContextManager::new();

    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            PipelineCommand::StartRecording => {
                {
                    let mut buf = audio_buf.lock().unwrap();
                    buf.0.clear();
                    buf.1 = true; // is_recording = true
                }
                let _ = result_tx.send(PipelineResult::RecordingStarted).await;
            }

            PipelineCommand::StopRecording => {
                // Drain accumulated audio
                let audio: Vec<f32> = {
                    let mut buf = audio_buf.lock().unwrap();
                    buf.1 = false; // is_recording = false
                    std::mem::take(&mut buf.0)
                };

                let duration_secs = audio.len() as f32 / 16_000.0;
                let _ = result_tx
                    .send(PipelineResult::RecordingStopped { duration_secs })
                    .await;

                // Guard: audio must be at least 0.5 s (8 000 samples)
                if audio.len() < 8_000 {
                    let _ = result_tx
                        .send(PipelineResult::Error {
                            message: "การบันทึกสั้นเกินไป (ต้องการอย่างน้อย 0.5 วินาที)"
                                .into(),
                        })
                        .await;
                    continue;
                }

                // --- STT (blocking → spawn_blocking) ----------------------
                let stt_clone = Arc::clone(&stt);
                let audio_clone = audio.clone();

                let stt_result = tokio::task::spawn_blocking(move || {
                    stt_clone.transcribe(&audio_clone)
                })
                .await;

                let raw_text = match stt_result {
                    Ok(Ok(text)) => text,
                    Ok(Err(e)) => {
                        let _ = result_tx
                            .send(PipelineResult::Error {
                                message: format!("STT ผิดพลาด: {e}"),
                            })
                            .await;
                        continue;
                    }
                    Err(e) => {
                        let _ = result_tx
                            .send(PipelineResult::Error {
                                message: format!("ข้อผิดพลาดภายใน: {e}"),
                            })
                            .await;
                        continue;
                    }
                };

                let _ = result_tx
                    .send(PipelineResult::TranscriptionComplete {
                        raw_text: raw_text.clone(),
                    })
                    .await;

                // --- LLM correction (if enabled) --------------------------
                let final_text = if config.llm.enabled
                    && config.operating_mode != voice_to_text::config::OperatingMode::Fast
                {
                    let context = ctx_mgr.build_context();
                    match llm.correct(&raw_text, context.as_deref()).await {
                        Ok(corrected) => {
                            ctx_mgr.push_sentence(corrected.clone());
                            let _ = result_tx
                                .send(PipelineResult::CorrectionComplete {
                                    corrected_text: corrected.clone(),
                                })
                                .await;
                            corrected
                        }
                        Err(e) => {
                            log::warn!("LLM correction failed (fallback to raw): {e}");
                            // Treat raw text as the result
                            let _ = result_tx
                                .send(PipelineResult::CorrectionComplete {
                                    corrected_text: raw_text.clone(),
                                })
                                .await;
                            raw_text.clone()
                        }
                    }
                } else {
                    // Fast mode: skip LLM
                    let _ = result_tx
                        .send(PipelineResult::CorrectionComplete {
                            corrected_text: raw_text.clone(),
                        })
                        .await;
                    raw_text.clone()
                };

                // --- Inject (blocking → spawn_blocking) -------------------
                let injector_clone = injector.clone();
                let text_clone = final_text.clone();
                let _inject_result = tokio::task::spawn_blocking(move || {
                    injector_clone.inject(&text_clone)
                })
                .await;

                let _ = result_tx.send(PipelineResult::InjectionComplete).await;
            }

            PipelineCommand::Cancel => {
                let mut buf = audio_buf.lock().unwrap();
                buf.1 = false;
                buf.0.clear();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Native options builder
// ---------------------------------------------------------------------------

fn native_options(config: &AppConfig) -> eframe::NativeOptions {
    let mut vp = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size([300.0, 120.0])
        .with_min_inner_size([250.0, 50.0])
        .with_resizable(false);

    if config.ui.always_on_top {
        vp = vp.with_always_on_top();
    }

    if let Some((x, y)) = config.ui.window_position {
        vp = vp.with_position(egui::pos2(x, y));
    }

    eframe::NativeOptions {
        viewport: vp,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() -> eframe::Result<()> {
    // 1. Logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Thai Voice-to-Text starting up");

    // 2. Configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        log::warn!("Failed to load config ({e}); using defaults");
        AppConfig::default()
    });

    // 3. Tokio runtime (2 worker threads — STT + LLM each take one)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    // 4. LLM corrector
    let llm: Arc<dyn LlmCorrector> = Arc::new(FallbackCorrector::new(
        ApiCorrector::from_config(&config.llm),
    ));

    // 5. Channel setup
    let (command_tx, command_rx) = mpsc::channel::<PipelineCommand>(16);
    let (result_tx, result_rx) = mpsc::channel::<PipelineResult>(32);
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<voice_to_text::hotkey::HotkeyEvent>(16);

    // 6. Shared audio buffer
    let audio_buf: SharedAudioBuffer = Arc::new(Mutex::new((Vec::new(), false)));

    // Build STT engine (may fail if model not present — degrade gracefully)
    let stt_model_path = voice_to_text::config::AppPaths::new()
        .models_dir
        .join(format!("{}.bin", config.stt.model));

    let stt_params = TranscribeParams {
        language: config.stt.language.clone(),
        ..TranscribeParams::default()
    };

    let stt: Arc<dyn SttEngine> = match WhisperEngine::load(&stt_model_path, stt_params) {
        Ok(engine) => {
            log::info!("Whisper model loaded: {}", stt_model_path.display());
            Arc::new(engine)
        }
        Err(e) => {
            log::warn!(
                "Could not load Whisper model ({}): {e}. STT will return an error.",
                stt_model_path.display()
            );
            // Use a stub that always returns an explanatory error so the app
            // still launches without a model file present.
            Arc::new(NoModelStt {
                path: stt_model_path.display().to_string(),
            })
        }
    };

    let injector = TextInjector::new();

    // Spawn pipeline orchestrator onto the tokio runtime
    {
        let audio_buf_clone = Arc::clone(&audio_buf);
        let stt_clone = Arc::clone(&stt);
        let llm_clone = Arc::clone(&llm);
        let injector_clone = injector.clone();
        let config_clone = config.clone();
        let result_tx_clone = result_tx.clone();

        rt.spawn(run_pipeline(
            audio_buf_clone,
            stt_clone,
            llm_clone,
            injector_clone,
            config_clone,
            command_rx,
            result_tx_clone,
        ));
    }

    // 7. Hotkey listener thread
    let hotkey_key = parse_key(&config.hotkey.push_to_talk_key)
        .unwrap_or(rdev::Key::F9);
    let _hotkey_listener = HotkeyListener::start(hotkey_key, hotkey_tx);

    // 8. cpal audio capture — pushes resampled mono samples into audio_buf
    //    when is_recording is true.
    let audio_buf_audio = Arc::clone(&audio_buf);
    let native_sample_rate;

    let _stream_handle: Option<voice_to_text::audio::StreamHandle> =
        match AudioCapture::new() {
            Ok(capture) => {
                native_sample_rate = capture.sample_rate();
                let channels = capture.channels();
                let (chunk_tx, chunk_rx) = std::sync::mpsc::channel::<AudioChunk>();

                // Spawn a thread that drains cpal chunks → resamples → feeds
                // the shared audio buffer (only when is_recording).
                std::thread::Builder::new()
                    .name("audio-resample".into())
                    .spawn(move || {
                        while let Ok(chunk) = chunk_rx.recv() {
                            // Check is_recording under a brief lock
                            let is_rec = audio_buf_audio.lock().unwrap().1;
                            if !is_rec {
                                continue;
                            }

                            // Downmix to mono
                            let mono = if channels > 1 {
                                voice_to_text::audio::stereo_to_mono(&chunk.samples, channels)
                            } else {
                                chunk.samples.clone()
                            };

                            // Resample to 16 kHz
                            let resampled = if chunk.sample_rate != 16_000 {
                                voice_to_text::audio::resample_to_16k(&mono, chunk.sample_rate)
                            } else {
                                mono
                            };

                            audio_buf_audio
                                .lock()
                                .unwrap()
                                .0
                                .extend_from_slice(&resampled);
                        }
                    })
                    .expect("failed to spawn audio-resample thread");

                match capture.start(chunk_tx) {
                    Ok(handle) => {
                        log::info!(
                            "Audio capture started ({} Hz, {} ch)",
                            native_sample_rate,
                            channels
                        );
                        Some(handle)
                    }
                    Err(e) => {
                        log::warn!("Failed to start audio stream: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Audio capture unavailable: {e}");
                None
            }
        };

    // 9. Build the egui app and run it (blocks until the window is closed)
    let app = ThaiSttApp::new(hotkey_rx, command_tx, result_rx, config.clone());
    let options = native_options(&config);

    eframe::run_native(
        "Thai STT",
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
}

// ---------------------------------------------------------------------------
// NoModelStt — fallback SttEngine when the model file is not present
// ---------------------------------------------------------------------------

struct NoModelStt {
    path: String,
}

impl voice_to_text::stt::SttEngine for NoModelStt {
    fn transcribe(&self, _audio: &[f32]) -> Result<String, voice_to_text::stt::SttError> {
        Err(voice_to_text::stt::SttError::ModelNotFound(
            self.path.clone(),
        ))
    }
}
