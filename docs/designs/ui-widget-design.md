# UI Widget Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** egui floating widget — layout, states, wireframes, interactions

---

## 1. Design Goals

- **Always-on-top** floating widget (ไม่บัง taskbar)
- **Minimal footprint** — เล็กกว่า 350×120px ตอน idle
- **Non-intrusive** — transparent background, ไม่มี title bar
- **Draggable** — user ย้ายตำแหน่งได้
- **Quick access** — hotkey toggle visibility, push-to-talk

---

## 2. Widget States & Wireframes

### 2.1 Idle State (Collapsed)

```
┌───────────────────────────┐
│  🎤  Thai STT    ─  ×    │  ← drag bar + minimize/close
│  F9: Push-to-talk         │
└───────────────────────────┘
  Size: 280×50px
```

### 2.2 Recording State

```
┌───────────────────────────────────┐
│  🔴  Recording...        ─  ×    │
│  ▁▃▅▇▅▃▁▃▅▇▆▃▁  3.2s            │  ← waveform + duration
│  [Release F9 to finish]           │
└───────────────────────────────────┘
  Size: 300×80px
```

### 2.3 Transcribing State

```
┌───────────────────────────────────┐
│  ⏳  Transcribing...      ─  ×    │
│  ████████░░░░  60%                │  ← progress bar
└───────────────────────────────────┘
  Size: 300×65px
```

### 2.4 Correcting State (Standard/Context Mode)

```
┌───────────────────────────────────┐
│  ✨  Correcting...        ─  ×    │
│  [raw] ผม เสร็จ งาน แล้ว ครับ    │  ← raw STT (gray, italic)
│  ⏳ Polishing...                   │
└───────────────────────────────────┘
  Size: 300×80px
```

### 2.5 Result State

```
┌───────────────────────────────────┐
│  ✅  Done (8.2s)          ─  ×    │
│  ผมเสร็จงานแล้ว จะส่งให้พรุ่งนี้   │  ← corrected text
│  [Copy]  [Edit]  [Inject ▶]      │
└───────────────────────────────────┘
  Size: 300×95px
```

### 2.6 Error State

```
┌───────────────────────────────────┐
│  ⚠️  Error                ─  ×    │
│  Ollama not running               │
│  [Retry]  [Use STT only]          │
└───────────────────────────────────┘
  Size: 300×80px
```

### 2.7 Settings Panel (Expanded)

```
┌───────────────────────────────────────┐
│  ⚙️  Settings              ─  ×      │
│                                       │
│  Mode:  ○ Fast  ● Standard  ○ Context│
│                                       │
│  STT Model:  [Thonburian Medium ▾]   │
│  LLM Model:  [Qwen2.5-3B       ▾]   │
│  Hotkey:     [F9              ▾]     │
│                                       │
│  [Manage Vocabulary]                  │
│  [About]  [Close]                     │
└───────────────────────────────────────┘
  Size: 320×250px
```

---

## 3. egui Implementation

### 3.1 Window Setup

```rust
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)     // no OS title bar
            .with_transparent(true)      // transparent background
            .with_inner_size([300.0, 80.0])
            .with_min_inner_size([250.0, 50.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Thai STT",
        options,
        Box::new(|cc| Ok(Box::new(ThaiSttApp::new(cc)))),
    )
}
```

### 3.2 App State

```rust
pub struct ThaiSttApp {
    // Pipeline state
    pipeline_state: PipelineState,
    raw_text: Option<String>,
    corrected_text: Option<String>,
    processing_time: Option<f32>,

    // UI state
    show_settings: bool,
    is_dragging: bool,
    waveform: Vec<f32>,

    // Configuration
    settings: AppSettings,

    // Channels to pipeline threads
    command_tx: mpsc::Sender<PipelineCommand>,
    result_rx: mpsc::Receiver<PipelineResult>,
}
```

### 3.3 Main UI Loop

```rust
impl eframe::App for ThaiSttApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for pipeline results (non-blocking)
        self.poll_results();

        // Request repaint at 30fps during recording (for waveform)
        if self.pipeline_state == PipelineState::Recording {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        // Custom window frame (no OS decorations)
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 230))
                .rounding(8.0)
                .inner_margin(8.0))
            .show(ctx, |ui| {
                self.draw_title_bar(ui, ctx);
                ui.separator();

                match &self.pipeline_state {
                    PipelineState::Idle => self.draw_idle(ui),
                    PipelineState::Recording => self.draw_recording(ui),
                    PipelineState::Transcribing { .. } => self.draw_transcribing(ui),
                    PipelineState::Correcting => self.draw_correcting(ui),
                    PipelineState::Injecting => self.draw_result(ui),
                    PipelineState::Error { .. } => self.draw_error(ui),
                }
            });
    }
}
```

### 3.4 Custom Title Bar (Draggable)

```rust
impl ThaiSttApp {
    fn draw_title_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // Status icon
            let icon = match &self.pipeline_state {
                PipelineState::Idle => "🎤",
                PipelineState::Recording => "🔴",
                PipelineState::Transcribing { .. } => "⏳",
                PipelineState::Correcting => "✨",
                PipelineState::Injecting => "✅",
                PipelineState::Error { .. } => "⚠️",
            };
            ui.label(icon);

            // Title (draggable area)
            let title_response = ui.label("Thai STT");
            if title_response.dragged() {
                // Move window by drag delta
                if let Some(pos) = ctx.input(|i| i.viewport().outer_rect) {
                    let delta = ctx.input(|i| i.pointer.delta());
                    ctx.send_viewport_cmd(
                        egui::ViewportCommand::OuterPosition(
                            pos.min + delta
                        )
                    );
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button
                if ui.small_button("×").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                // Settings button
                if ui.small_button("⚙").clicked() {
                    self.show_settings = !self.show_settings;
                }
                // Minimize button
                if ui.small_button("─").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            });
        });
    }
}
```

### 3.5 Waveform Visualization

```rust
impl ThaiSttApp {
    fn draw_waveform(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 30.0),
            egui::Sense::hover(),
        );

        let painter = ui.painter();
        let num_bars = 40;
        let bar_width = rect.width() / num_bars as f32;

        for (i, &amplitude) in self.waveform.iter().take(num_bars).enumerate() {
            let x = rect.left() + i as f32 * bar_width;
            let bar_height = amplitude * rect.height();
            let center_y = rect.center().y;

            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(x + bar_width / 2.0, center_y),
                    egui::vec2(bar_width * 0.6, bar_height),
                ),
                2.0,
                egui::Color32::from_rgb(80, 200, 120), // green bars
            );
        }
    }
}
```

---

## 4. Color Scheme

| Element | Color | Hex |
|---------|-------|-----|
| Background | Dark gray (semi-transparent) | `#1E1E1E` alpha 90% |
| Text (primary) | White | `#FFFFFF` |
| Text (raw STT) | Gray italic | `#888888` |
| Text (corrected) | White | `#FFFFFF` |
| Recording indicator | Red | `#FF4444` |
| Waveform bars | Green | `#50C878` |
| Progress bar | Blue | `#4488FF` |
| Error text | Orange | `#FF8844` |
| Button | Subtle gray | `#3A3A3A` |
| Button (hover) | Lighter gray | `#4A4A4A` |

---

## 5. Interaction Design

### 5.1 Push-to-Talk Flow

```
User Action                Widget Response
──────────────────────────────────────────────
Hold F9                 →  Start recording, show waveform
                           Widget changes to Recording state
Release F9              →  Stop recording, start STT
                           Widget changes to Transcribing
STT complete            →  Show raw text (gray)
                           Start LLM correction (if not Fast Mode)
LLM complete            →  Show corrected text (white)
                           Auto-inject to active window
3s timeout              →  Return to Idle
```

### 5.2 Manual Actions

| Action | Trigger | Behavior |
|--------|---------|----------|
| Copy text | Click [Copy] | Copy corrected text to clipboard |
| Edit text | Click [Edit] | Show editable text field |
| Re-inject | Click [Inject] | Inject text again |
| Settings | Click ⚙ | Toggle settings panel |
| Move widget | Drag title area | Move widget position |
| Dismiss result | Click anywhere / 5s | Return to Idle |

### 5.3 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| F9 (hold) | Push-to-talk (configurable) |
| Escape | Cancel current operation |
| Ctrl+Shift+T | Toggle widget visibility |

---

## 6. Responsive Sizing

```rust
impl ThaiSttApp {
    fn update_window_size(&self, ctx: &egui::Context) {
        let size = match &self.pipeline_state {
            PipelineState::Idle => egui::vec2(280.0, 50.0),
            PipelineState::Recording => egui::vec2(300.0, 80.0),
            PipelineState::Transcribing { .. } => egui::vec2(300.0, 65.0),
            PipelineState::Correcting => egui::vec2(300.0, 80.0),
            PipelineState::Injecting => egui::vec2(300.0, 95.0),
            PipelineState::Error { .. } => egui::vec2(300.0, 80.0),
        };

        ctx.send_viewport_cmd(
            egui::ViewportCommand::InnerSize(size)
        );
    }
}
```

---

## 7. System Tray Integration (Phase 4)

```
สำหรับ Phase 4:
- เพิ่ม system tray icon
- Right-click menu: Show/Hide, Settings, Quit
- Double-click: Toggle widget visibility
- ใช้ crate: tray-icon หรือ tao
```

---

## 8. Dependencies

```toml
[dependencies]
eframe = "0.31"
egui = "0.31"

# Phase 4:
# tray-icon = "0.19"  # System tray
```
