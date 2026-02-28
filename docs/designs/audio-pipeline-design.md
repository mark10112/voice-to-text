# Audio Pipeline Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** Microphone capture → Resampling → VAD → Ring Buffer

---

## 1. Pipeline Overview

```
┌────────────┐    ┌─────────────┐    ┌──────────┐    ┌────────────┐
│ Microphone │───▶│  cpal       │───▶│ Resample │───▶│ Ring Buffer│
│            │    │  callback   │    │ to 16kHz │    │ (f32 mono) │
└────────────┘    └─────────────┘    └──────────┘    └─────┬──────┘
                                                           │
                                                           ▼
                                                     ┌──────────┐
                                                     │  VAD     │
                                                     │  (trim)  │
                                                     └──────────┘
```

---

## 2. Audio Capture (cpal)

### 2.1 Device Selection

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioCapture {
    device: cpal::Device,
    config: cpal::StreamConfig,
    stream: Option<cpal::Stream>,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device found")?;

        // ใช้ config ที่ใกล้ 16kHz ที่สุด
        let supported = device.supported_input_configs()?;
        let config = Self::select_best_config(supported)?;

        Ok(Self { device, config, stream: None })
    }
}
```

### 2.2 Target Format

| Parameter | Value | เหตุผล |
|-----------|-------|--------|
| Sample Rate | 16,000 Hz | Whisper ต้องการ 16kHz |
| Channels | 1 (Mono) | STT ไม่ต้องการ stereo |
| Sample Format | f32 | whisper-rs รับ `&[f32]` |
| Bit Depth | 32-bit float | Range: -1.0 to 1.0 |

### 2.3 Stream Creation

```rust
pub fn start_recording(&mut self, buffer: Arc<Mutex<AudioBuffer>>) -> Result<()> {
    let config = self.config.clone();

    let stream = self.device.build_input_stream(
        &config,
        move |data: &[f32], _info: &cpal::InputCallbackInfo| {
            // Callback: ทำงานบน audio thread
            // ส่ง samples เข้า ring buffer
            if let Ok(mut buf) = buffer.lock() {
                buf.push_samples(data);
            }
        },
        |err| {
            eprintln!("Audio stream error: {}", err);
        },
        None, // no timeout
    )?;

    stream.play()?;
    self.stream = Some(stream);
    Ok(())
}

pub fn stop_recording(&mut self) {
    self.stream = None; // Drop stream → stops capture
}
```

### 2.4 Platform-Specific Notes

| OS | Audio Backend | หมายเหตุ |
|----|--------------|----------|
| Windows | WASAPI | Default host ใช้ได้เลย ไม่ต้องติดตั้งเพิ่ม |
| macOS | CoreAudio | ต้องขอ microphone permission ครั้งแรก |
| Linux | PulseAudio / PipeWire | ต้องติดตั้ง `libasound2-dev` สำหรับ build |

---

## 3. Resampling

### 3.1 ทำไมต้อง Resample

- Microphone ส่วนใหญ่ output 44,100 Hz หรือ 48,000 Hz
- Whisper ต้องการ 16,000 Hz
- ต้อง resample ก่อนส่งให้ STT

### 3.2 Strategy

```rust
pub fn resample_to_16k(samples: &[f32], source_rate: u32) -> Vec<f32> {
    if source_rate == 16000 {
        return samples.to_vec();
    }

    let ratio = 16000.0 / source_rate as f64;
    let output_len = (samples.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        // Linear interpolation
        let sample = if idx + 1 < samples.len() {
            samples[idx] as f64 * (1.0 - frac) + samples[idx + 1] as f64 * frac
        } else {
            samples[idx] as f64
        };

        output.push(sample as f32);
    }

    output
}
```

### 3.3 ทางเลือก Crate

| Crate | ข้อดี | ข้อเสีย |
|-------|--------|---------|
| Manual (linear interp) | ไม่มี dependency เพิ่ม | คุณภาพต่ำ |
| `rubato` | High-quality sinc resampling | เพิ่ม dependency |
| `dasp` | Comprehensive DSP | อาจหนักเกิน |

**แนะนำ:** เริ่มด้วย linear interpolation (MVP) → เปลี่ยนเป็น `rubato` ถ้าคุณภาพเสียงไม่ดี

---

## 4. Ring Buffer

### 4.1 Design

```rust
pub struct AudioBuffer {
    data: Vec<f32>,
    capacity: usize,
    write_pos: usize,
    is_recording: bool,
}

impl AudioBuffer {
    /// สร้าง buffer สำหรับ max_seconds วินาที ที่ 16kHz
    pub fn new(max_seconds: usize) -> Self {
        let capacity = max_seconds * 16_000;
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
            is_recording: false,
        }
    }

    /// เพิ่ม samples จาก cpal callback
    pub fn push_samples(&mut self, samples: &[f32]) {
        if !self.is_recording { return; }

        for &sample in samples {
            if self.data.len() < self.capacity {
                self.data.push(sample);
            }
            // ถ้าเต็ม → หยุดเก็บ (ไม่ overwrite สำหรับ push-to-talk)
        }
    }

    /// ดึง audio ทั้งหมดออกมา แล้ว clear buffer
    pub fn drain(&mut self) -> Vec<f32> {
        let audio = std::mem::take(&mut self.data);
        self.write_pos = 0;
        audio
    }

    /// Clear buffer (เริ่ม recording ใหม่)
    pub fn clear(&mut self) {
        self.data.clear();
        self.write_pos = 0;
    }

    pub fn duration_seconds(&self) -> f32 {
        self.data.len() as f32 / 16_000.0
    }
}
```

### 4.2 Buffer Sizing

| Use Case | Max Duration | Buffer Size | Memory |
|----------|-------------|-------------|--------|
| Short utterance | 10s | 160,000 samples | ~625 KB |
| Medium dictation | 30s | 480,000 samples | ~1.9 MB |
| Long dictation | 60s | 960,000 samples | ~3.75 MB |

**Default:** 60 วินาที (~4 MB) — เพียงพอสำหรับ push-to-talk ทุกกรณี

---

## 5. Voice Activity Detection (VAD)

### 5.1 Purpose

ตัด silence ที่ต้นและท้ายเสียงออกก่อนส่ง Whisper:
- ลดเวลาประมวลผล (ไม่ต้อง transcribe silence)
- ลด hallucination (Whisper อาจ hallucinate ในช่วง silence)

### 5.2 Simple Energy-Based VAD (MVP)

```rust
pub fn trim_silence(audio: &[f32], threshold: f32) -> &[f32] {
    let frame_size = 480; // 30ms at 16kHz

    // หา start: frame แรกที่เสียงดังพอ
    let start = audio.chunks(frame_size)
        .position(|chunk| {
            let energy = chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
            energy > threshold
        })
        .unwrap_or(0) * frame_size;

    // หา end: frame สุดท้ายที่เสียงดังพอ
    let end = audio.chunks(frame_size)
        .rposition(|chunk| {
            let energy = chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32;
            energy > threshold
        })
        .map(|pos| (pos + 1) * frame_size)
        .unwrap_or(audio.len());

    &audio[start..end.min(audio.len())]
}
```

### 5.3 Advanced VAD (Phase 2+)

whisper-rs มี built-in VAD ผ่าน `WhisperVadSegments`:
- ใช้ Silero VAD model (ONNX)
- ตรวจจับ speech segments อัตโนมัติ
- แม่นยำกว่า energy-based approach

```rust
// whisper-rs VAD (ถ้ารองรับใน version ที่ใช้)
let vad_params = WhisperVadParams::default();
let segments = state.vad(&audio_data, &vad_params)?;

for i in 0..segments.num_segments() {
    let start = segments.get_segment_start_timestamp(i);
    let end = segments.get_segment_end_timestamp(i);
    // Process each speech segment
}
```

---

## 6. Audio Quality Validation

### 6.1 Pre-Transcription Checks

```rust
pub struct AudioValidator;

impl AudioValidator {
    /// ตรวจสอบ audio ก่อนส่ง STT
    pub fn validate(audio: &[f32]) -> Result<(), AudioError> {
        // 1. ความยาวขั้นต่ำ (0.5 วินาที)
        if audio.len() < 8_000 {
            return Err(AudioError::TooShort);
        }

        // 2. ตรวจ silence ทั้งหมด
        let max_amplitude = audio.iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);

        if max_amplitude < 0.01 {
            return Err(AudioError::TooQuiet);
        }

        // 3. ตรวจ clipping
        let clipped_samples = audio.iter()
            .filter(|s| s.abs() > 0.99)
            .count();

        if clipped_samples as f32 / audio.len() as f32 > 0.1 {
            return Err(AudioError::Clipping);
        }

        Ok(())
    }
}

pub enum AudioError {
    TooShort,
    TooQuiet,
    Clipping,
}
```

---

## 7. Waveform Data for UI

Widget ต้องการ waveform data สำหรับแสดงผลขณะ recording:

```rust
/// คำนวณ RMS amplitude สำหรับแสดง waveform ใน UI
pub fn compute_waveform(audio: &[f32], num_bars: usize) -> Vec<f32> {
    let chunk_size = audio.len() / num_bars;
    if chunk_size == 0 { return vec![0.0; num_bars]; }

    audio.chunks(chunk_size)
        .take(num_bars)
        .map(|chunk| {
            let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
            rms.min(1.0) // clamp to 0.0-1.0
        })
        .collect()
}
```

---

## 8. Dependencies

```toml
[dependencies]
cpal = "0.15"      # Cross-platform audio capture

# Optional (Phase 2):
# rubato = "0.15"  # High-quality resampling
# hound = "3.5"    # WAV file I/O (for debug/testing)
```
