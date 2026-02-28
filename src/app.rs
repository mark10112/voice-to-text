//! Thai STT floating widget — egui/eframe application.
//!
//! # Architecture
//!
//! [`ThaiSttApp`] is the top-level [`eframe::App`] that owns the UI state and
//! two channel endpoints:
//!
//! * `command_tx` — sends [`PipelineCommand`] to the pipeline orchestrator.
//! * `result_rx`  — receives [`PipelineResult`] from the orchestrator and
//!   hotkey events forwarded by the main-thread poll.
//!
//! The app renders a compact, always-on-top, borderless, transparent floating
//! widget whose appearance changes to match the current [`PipelineState`].
//!
//! # Widget States
//!
//! | State | Visual |
//! |-------|--------|
//! | `Idle` | "กด F9 เพื่อพูด" — dim gray |
//! | `Recording` | Waveform bars + elapsed timer — red indicator |
//! | `Transcribing` | Spinner + "กำลังแปลงเสียง..." |
//! | `Correcting` | Spinner + "กำลังปรับปรุง..." |
//! | `Result` | Corrected text — green, auto-clear after 3 s |
//! | `Error` | Error message — orange |

use std::time::{Duration, Instant};

use eframe::egui;
use tokio::sync::mpsc;

use crate::config::AppConfig;

// ---------------------------------------------------------------------------
// Pipeline message types (owned by the ui module until the pipeline crate
// is merged; the orchestrator imports them from here).
// ---------------------------------------------------------------------------

/// Commands sent from the UI thread to the pipeline orchestrator.
#[derive(Debug, Clone)]
pub enum PipelineCommand {
    /// Start capturing audio.
    StartRecording,
    /// Stop capturing audio and begin transcription.
    StopRecording,
    /// Abort the current operation and return to idle.
    Cancel,
}

/// Results / progress events delivered from the pipeline to the UI.
#[derive(Debug, Clone)]
pub enum PipelineResult {
    /// The pipeline acknowledged the start-recording command.
    RecordingStarted,
    /// Recording has stopped; `duration_secs` is how long the user held the
    /// push-to-talk key.
    RecordingStopped { duration_secs: f32 },
    /// STT completed; `raw_text` is the unmodified Whisper output.
    TranscriptionComplete { raw_text: String },
    /// LLM correction completed.
    CorrectionComplete { corrected_text: String },
    /// Text injection finished — pipeline is idle again.
    InjectionComplete,
    /// A waveform snapshot for the recording animation (sent ~30 fps).
    WaveformUpdate { bars: Vec<f32> },
    /// An error occurred at any pipeline stage.
    Error { message: String },
}

// ---------------------------------------------------------------------------
// PipelineState — UI-side state machine
// ---------------------------------------------------------------------------

/// Current state of the STT pipeline, as seen by the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
    /// Waiting for the user to press the push-to-talk hotkey.
    Idle,
    /// Audio is being captured.
    Recording,
    /// Whisper is running inference on the recorded audio.
    Transcribing,
    /// LLM is correcting the raw transcript.
    Correcting,
    /// The final result is ready and is being displayed.
    Result,
    /// An error occurred.
    Error,
}

// ---------------------------------------------------------------------------
// ThaiSttApp
// ---------------------------------------------------------------------------

/// eframe application — the floating Thai STT widget.
pub struct ThaiSttApp {
    // ── Pipeline state ───────────────────────────────────────────────────
    /// Current logical state of the processing pipeline.
    pub pipeline_state: PipelineState,
    /// Raw (pre-LLM) transcript text, shown during the Correcting state.
    pub raw_text: Option<String>,
    /// Final corrected text, shown in the Result state.
    pub corrected_text: Option<String>,
    /// Human-readable error message for the Error state.
    pub error_message: Option<String>,

    // ── Timing ───────────────────────────────────────────────────────────
    /// When the current recording started (used for the elapsed-time display).
    recording_start: Option<Instant>,
    /// When the Result state was entered (used for the 3 s auto-clear timer).
    result_time: Option<Instant>,

    // ── Waveform ─────────────────────────────────────────────────────────
    /// Amplitude bars for the waveform visualisation during recording.
    waveform: Vec<f32>,

    // ── UI state ─────────────────────────────────────────────────────────
    /// Whether the settings panel is expanded.
    show_settings: bool,
    /// Spinner animation phase (increases each frame).
    spinner_phase: f32,

    // ── Channels ─────────────────────────────────────────────────────────
    /// Send commands to the background pipeline orchestrator.
    pub command_tx: mpsc::Sender<PipelineCommand>,
    /// Receive results / progress from the background pipeline orchestrator.
    pub result_rx: mpsc::Receiver<PipelineResult>,
    /// Receive hotkey events forwarded by main (hotkey thread → UI).
    pub hotkey_rx: mpsc::Receiver<crate::hotkey::HotkeyEvent>,

    // ── Configuration ────────────────────────────────────────────────────
    /// Application configuration (read-only after startup).
    pub config: AppConfig,
}

impl ThaiSttApp {
    /// Create a new [`ThaiSttApp`].
    ///
    /// * `hotkey_rx`  — receiver end of the hotkey channel.
    /// * `command_tx` — sender end of the pipeline command channel.
    /// * `result_rx`  — receiver end of the pipeline result channel.
    /// * `config`     — loaded application configuration.
    pub fn new(
        hotkey_rx: mpsc::Receiver<crate::hotkey::HotkeyEvent>,
        command_tx: mpsc::Sender<PipelineCommand>,
        result_rx: mpsc::Receiver<PipelineResult>,
        config: AppConfig,
    ) -> Self {
        Self {
            pipeline_state: PipelineState::Idle,
            raw_text: None,
            corrected_text: None,
            error_message: None,
            recording_start: None,
            result_time: None,
            waveform: vec![0.0; 30],
            show_settings: false,
            spinner_phase: 0.0,
            command_tx,
            result_rx,
            hotkey_rx,
            config,
        }
    }

    // ── Channel polling ──────────────────────────────────────────────────

    /// Drain all pending hotkey events (non-blocking).
    fn poll_hotkey(&mut self) {
        while let Ok(event) = self.hotkey_rx.try_recv() {
            match event {
                crate::hotkey::HotkeyEvent::PushToTalkPressed => {
                    if self.pipeline_state == PipelineState::Idle {
                        self.pipeline_state = PipelineState::Recording;
                        self.recording_start = Some(Instant::now());
                        self.waveform = vec![0.0; 30];
                        let _ = self.command_tx.try_send(PipelineCommand::StartRecording);
                    }
                }
                crate::hotkey::HotkeyEvent::PushToTalkReleased => {
                    if self.pipeline_state == PipelineState::Recording {
                        self.pipeline_state = PipelineState::Transcribing;
                        let _ = self.command_tx.try_send(PipelineCommand::StopRecording);
                    }
                }
                crate::hotkey::HotkeyEvent::ToggleVisibility => {
                    // Visibility toggling is handled via ViewportCommand; no
                    // state change is needed here.
                }
            }
        }
    }

    /// Drain all pending pipeline results (non-blocking).
    fn poll_results(&mut self) {
        while let Ok(result) = self.result_rx.try_recv() {
            match result {
                PipelineResult::RecordingStarted => {
                    // State was already set to Recording on hotkey press;
                    // this is just a confirmation.
                }
                PipelineResult::RecordingStopped { .. } => {
                    // State was already set to Transcribing on hotkey release.
                }
                PipelineResult::WaveformUpdate { bars } => {
                    self.waveform = bars;
                }
                PipelineResult::TranscriptionComplete { raw_text } => {
                    self.raw_text = Some(raw_text);
                    self.pipeline_state = PipelineState::Correcting;
                }
                PipelineResult::CorrectionComplete { corrected_text } => {
                    self.corrected_text = Some(corrected_text);
                    self.pipeline_state = PipelineState::Result;
                    self.result_time = Some(Instant::now());
                }
                PipelineResult::InjectionComplete => {
                    // Already in Result; will auto-clear after timeout.
                }
                PipelineResult::Error { message } => {
                    self.error_message = Some(message);
                    self.pipeline_state = PipelineState::Error;
                }
            }
        }
    }

    /// Auto-clear the Result state after 3 seconds.
    fn check_result_timeout(&mut self) {
        if self.pipeline_state == PipelineState::Result {
            if let Some(t) = self.result_time {
                if t.elapsed() >= Duration::from_secs(3) {
                    self.reset_to_idle();
                }
            }
        }
    }

    /// Reset all transient state and return to Idle.
    fn reset_to_idle(&mut self) {
        self.pipeline_state = PipelineState::Idle;
        self.raw_text = None;
        self.corrected_text = None;
        self.error_message = None;
        self.recording_start = None;
        self.result_time = None;
        self.waveform = vec![0.0; 30];
    }

    // ── Window sizing ────────────────────────────────────────────────────

    /// Resize the window to match the current pipeline state.
    fn update_window_size(&self, ctx: &egui::Context) {
        let size = match &self.pipeline_state {
            PipelineState::Idle => egui::vec2(280.0, 50.0),
            PipelineState::Recording => egui::vec2(300.0, 80.0),
            PipelineState::Transcribing => egui::vec2(300.0, 65.0),
            PipelineState::Correcting => egui::vec2(300.0, 80.0),
            PipelineState::Result => egui::vec2(300.0, 95.0),
            PipelineState::Error => egui::vec2(300.0, 80.0),
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    }

    // ── Custom title bar ─────────────────────────────────────────────────

    /// Draw the draggable title bar with status icon, title, and window
    /// controls (settings, minimise, close).
    fn draw_title_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // Status icon
            let icon = match &self.pipeline_state {
                PipelineState::Idle => "  ",
                PipelineState::Recording => "* ",
                PipelineState::Transcribing => ". ",
                PipelineState::Correcting => "~ ",
                PipelineState::Result => "OK",
                PipelineState::Error => "! ",
            };
            ui.label(egui::RichText::new(icon).color(self.state_color()));

            // Draggable title area
            let title_resp = ui.label(
                egui::RichText::new("Thai STT")
                    .color(egui::Color32::from_rgb(200, 200, 200))
                    .size(13.0),
            );
            if title_resp.is_pointer_button_down_on() {
                if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                    let delta = ctx.input(|i| i.pointer.delta());
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                        outer_rect.min + delta,
                    ));
                }
            }

            // Right-aligned window controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("x")
                                .color(egui::Color32::from_rgb(200, 100, 100))
                                .size(12.0),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                // Minimise
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("-")
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .size(12.0),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
                // Settings toggle
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("=")
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .size(12.0),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    self.show_settings = !self.show_settings;
                }
            });
        });
    }

    // ── State-specific panel renderers ───────────────────────────────────

    /// Render the Idle state panel: hotkey hint.
    fn draw_idle(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new("กด F9 เพื่อพูด")
                    .color(egui::Color32::from_rgb(120, 120, 120))
                    .size(13.0),
            );
        });
    }

    /// Render the Recording state panel: waveform + elapsed timer.
    fn draw_recording(&mut self, ui: &mut egui::Ui) {
        let elapsed = self
            .recording_start
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        ui.add_space(4.0);

        // Elapsed timer
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("กำลังบันทึก")
                    .color(egui::Color32::from_rgb(255, 80, 80))
                    .size(12.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1}s", elapsed))
                        .color(egui::Color32::from_rgb(255, 140, 140))
                        .size(12.0),
                );
            });
        });

        ui.add_space(4.0);
        self.draw_waveform(ui);

        ui.add_space(2.0);
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new("ปล่อย F9 เพื่อหยุด")
                    .color(egui::Color32::from_rgb(160, 160, 160))
                    .size(10.0),
            );
        });
    }

    /// Render the Transcribing state panel: spinner + label.
    fn draw_transcribing(&self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(format!("{} กำลังแปลงเสียง...", self.spinner_char()))
                    .color(egui::Color32::from_rgb(68, 136, 255))
                    .size(13.0),
            );
        });
    }

    /// Render the Correcting state panel: raw text hint + spinner.
    fn draw_correcting(&self, ui: &mut egui::Ui) {
        if let Some(ref raw) = self.raw_text {
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(raw.as_str())
                    .color(egui::Color32::from_rgb(130, 130, 130))
                    .italics()
                    .size(11.0),
            );
            ui.add_space(2.0);
        } else {
            ui.add_space(8.0);
        }
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(format!("{} กำลังปรับปรุง...", self.spinner_char()))
                    .color(egui::Color32::from_rgb(68, 136, 255))
                    .size(13.0),
            );
        });
    }

    /// Render the Result state panel: corrected text + action buttons.
    fn draw_result(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let text = self
            .corrected_text
            .clone()
            .unwrap_or_else(|| self.raw_text.clone().unwrap_or_default());

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(text.as_str())
                .color(egui::Color32::from_rgb(80, 200, 120))
                .size(13.0),
        );

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("คัดลอก").size(11.0),
                ))
                .clicked()
            {
                ctx.copy_text(text.clone());
            }
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("ปิด").size(11.0),
                ))
                .clicked()
            {
                self.reset_to_idle();
            }
        });
    }

    /// Render the Error state panel: message + retry / dismiss buttons.
    fn draw_error(&mut self, ui: &mut egui::Ui) {
        let msg = self
            .error_message
            .clone()
            .unwrap_or_else(|| "ข้อผิดพลาดที่ไม่ทราบสาเหตุ".into());

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(msg.as_str())
                .color(egui::Color32::from_rgb(255, 136, 68))
                .size(12.0),
        );

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("ปิด").size(11.0),
                ))
                .clicked()
            {
                self.reset_to_idle();
            }
            if ui
                .add(egui::Button::new(
                    egui::RichText::new("ยกเลิก").size(11.0),
                ))
                .clicked()
            {
                let _ = self.command_tx.try_send(PipelineCommand::Cancel);
                self.reset_to_idle();
            }
        });
    }

    /// Render the settings panel.
    fn draw_settings(&self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("โหมด:")
                .color(egui::Color32::from_rgb(180, 180, 180))
                .size(12.0),
        );
        ui.label(
            egui::RichText::new(format!("  {:?}", self.config.operating_mode))
                .color(egui::Color32::from_rgb(140, 140, 140))
                .size(11.0),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(format!("  STT: {}", self.config.stt.model))
                .color(egui::Color32::from_rgb(140, 140, 140))
                .size(11.0),
        );
        ui.label(
            egui::RichText::new(format!("  LLM: {}", self.config.llm.model))
                .color(egui::Color32::from_rgb(140, 140, 140))
                .size(11.0),
        );
        ui.label(
            egui::RichText::new(format!(
                "  Hotkey: {}",
                self.config.hotkey.push_to_talk_key
            ))
            .color(egui::Color32::from_rgb(140, 140, 140))
            .size(11.0),
        );
    }

    // ── Waveform helper ───────────────────────────────────────────────────

    /// Draw the amplitude bar chart used in the Recording state.
    fn draw_waveform(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 28.0),
            egui::Sense::hover(),
        );

        let painter = ui.painter();
        let num_bars = self.waveform.len().max(1);
        let bar_width = rect.width() / num_bars as f32;

        for (i, &amplitude) in self.waveform.iter().enumerate() {
            let x = rect.left() + i as f32 * bar_width;
            let bar_height = (amplitude * rect.height()).max(2.0);
            let center_y = rect.center().y;

            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(x + bar_width / 2.0, center_y),
                    egui::vec2((bar_width * 0.65).max(1.0), bar_height),
                ),
                1.0,
                egui::Color32::from_rgb(80, 200, 120),
            );
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    /// A simple rotating ASCII spinner character driven by `spinner_phase`.
    fn spinner_char(&self) -> char {
        let chars = ['|', '/', '-', '\\'];
        let idx = (self.spinner_phase as usize) % chars.len();
        chars[idx]
    }

    /// Primary accent colour for the current state (used in the title bar icon).
    fn state_color(&self) -> egui::Color32 {
        match &self.pipeline_state {
            PipelineState::Idle => egui::Color32::from_rgb(100, 100, 100),
            PipelineState::Recording => egui::Color32::from_rgb(255, 68, 68),
            PipelineState::Transcribing => egui::Color32::from_rgb(68, 136, 255),
            PipelineState::Correcting => egui::Color32::from_rgb(68, 136, 255),
            PipelineState::Result => egui::Color32::from_rgb(80, 200, 120),
            PipelineState::Error => egui::Color32::from_rgb(255, 136, 68),
        }
    }
}

// ---------------------------------------------------------------------------
// eframe::App impl
// ---------------------------------------------------------------------------

impl eframe::App for ThaiSttApp {
    /// Called every frame by eframe.  Polls channels, advances timers, then
    /// renders the widget.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Poll non-blocking channels ------------------------------------
        self.poll_hotkey();
        self.poll_results();
        self.check_result_timeout();

        // --- Advance spinner animation -------------------------------------
        self.spinner_phase += 0.08;
        if self.spinner_phase >= 4.0 {
            self.spinner_phase = 0.0;
        }

        // --- Schedule repaints while animated states are active -----------
        match &self.pipeline_state {
            PipelineState::Recording => {
                // Repaint at ~30 fps for the waveform animation
                ctx.request_repaint_after(Duration::from_millis(33));
            }
            PipelineState::Transcribing | PipelineState::Correcting => {
                // Repaint at ~15 fps for the spinner
                ctx.request_repaint_after(Duration::from_millis(66));
            }
            PipelineState::Result => {
                // Poll the auto-clear timeout at 1 fps
                ctx.request_repaint_after(Duration::from_millis(500));
            }
            _ => {}
        }

        // --- Resize window to match state ---------------------------------
        self.update_window_size(ctx);

        // --- Dark transparent background frame ----------------------------
        let frame = egui::Frame::new()
            .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(8));

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.draw_title_bar(ui, ctx);

            if self.show_settings {
                ui.separator();
                self.draw_settings(ui);
                return;
            }

            ui.separator();

            // Clone state to avoid borrow-check issues when calling &mut self
            // methods that also reference self.pipeline_state.
            let state = self.pipeline_state.clone();
            match state {
                PipelineState::Idle => self.draw_idle(ui),
                PipelineState::Recording => self.draw_recording(ui),
                PipelineState::Transcribing => self.draw_transcribing(ui),
                PipelineState::Correcting => self.draw_correcting(ui),
                PipelineState::Result => {
                    let ctx_clone = ctx.clone();
                    self.draw_result(ui, &ctx_clone);
                }
                PipelineState::Error => self.draw_error(ui),
            }
        });
    }

    /// Persist window position in the config on exit (best-effort).
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("Thai STT widget closing");
    }
}
