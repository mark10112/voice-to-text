# Text Injection Design

**วันที่:** 28 กุมภาพันธ์ 2026
**ขอบเขต:** Clipboard → paste injection, platform edge cases, Thai character handling

---

## 1. Injection Strategy

### 1.1 ทำไมใช้ Clipboard + Paste

ภาษาไทยมี combining characters (สระลอย, วรรณยุกต์) ที่ simulate keyboard ยาก:
- `ก` + `่` + `า` = `ก่า` — ต้องส่ง 3 key events ตามลำดับ
- Keyboard layout อาจไม่ตรงกัน
- บาง app จัดการ combining chars ต่างกัน

**Clipboard + Ctrl+V** หลีกเลี่ยงปัญหาทั้งหมด:
- ข้อความ Unicode ถูกต้องเสมอ
- ไม่ขึ้นกับ keyboard layout
- ทำงานได้กับทุก app ที่รับ paste

### 1.2 Flow

```
┌────────────────┐    ┌──────────────┐    ┌───────────────┐    ┌────────────┐
│ 1. Save        │───▶│ 2. Set       │───▶│ 3. Simulate   │───▶│ 4. Restore │
│ original       │    │ corrected    │    │ Ctrl+V        │    │ original   │
│ clipboard      │    │ text to      │    │ (or Cmd+V)    │    │ clipboard  │
│                │    │ clipboard    │    │               │    │ (optional) │
└────────────────┘    └──────────────┘    └───────────────┘    └────────────┘
```

---

## 2. Implementation

### 2.1 Core Injector

```rust
use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};

pub struct TextInjector {
    delay_ms: u64,  // delay ระหว่าง clipboard set กับ paste
}

impl TextInjector {
    pub fn new() -> Self {
        Self { delay_ms: 50 }
    }

    pub fn inject(&self, text: &str) -> Result<(), InjectError> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| InjectError::ClipboardAccess(e.to_string()))?;

        // 1. Save original clipboard content
        let original = clipboard.get_text().ok();

        // 2. Set new text
        clipboard.set_text(text)
            .map_err(|e| InjectError::ClipboardSet(e.to_string()))?;

        // 3. Small delay to ensure clipboard is ready
        std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));

        // 4. Simulate paste
        self.simulate_paste()?;

        // 5. Restore original clipboard (optional, after delay)
        if let Some(original_text) = original {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = clipboard.set_text(original_text);
        }

        Ok(())
    }

    fn simulate_paste(&self) -> Result<(), InjectError> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| InjectError::KeySimulation(e.to_string()))?;

        #[cfg(target_os = "macos")]
        {
            // macOS: Cmd+V
            enigo.key(Key::Meta, Direction::Press)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
            enigo.key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
            enigo.key(Key::Meta, Direction::Release)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Windows / Linux: Ctrl+V
            enigo.key(Key::Control, Direction::Press)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
            enigo.key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
            enigo.key(Key::Control, Direction::Release)
                .map_err(|e| InjectError::KeySimulation(e.to_string()))?;
        }

        Ok(())
    }
}
```

### 2.2 Error Types

```rust
#[derive(Debug)]
pub enum InjectError {
    ClipboardAccess(String),
    ClipboardSet(String),
    KeySimulation(String),
    TargetWindowLost,
}

impl std::fmt::Display for InjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClipboardAccess(e) => write!(f, "Cannot access clipboard: {}", e),
            Self::ClipboardSet(e) => write!(f, "Cannot set clipboard text: {}", e),
            Self::KeySimulation(e) => write!(f, "Cannot simulate key press: {}", e),
            Self::TargetWindowLost => write!(f, "Target window lost focus"),
        }
    }
}
```

---

## 3. Platform-Specific Considerations

### 3.1 Windows

| Issue | รายละเอียด | วิธีแก้ |
|-------|-----------|---------|
| UAC elevation | Admin apps ไม่รับ input จาก user-level apps | แจ้ง user ว่าต้อง run as admin (rare) |
| Antivirus | Windows Defender อาจ flag key simulation | Code-sign binary |
| Focus timing | App อาจต้อง re-focus หลัง paste | เพิ่ม delay 50ms |

### 3.2 macOS

| Issue | รายละเอียด | วิธีแก้ |
|-------|-----------|---------|
| Accessibility permission | ต้องขอ Accessibility permission | First-run prompt + instructions |
| Cmd vs Ctrl | macOS ใช้ Cmd+V | Compile-time `#[cfg]` check |
| Sandboxed apps | บาง app restrict paste | ไม่มีทางแก้ — document limitation |

### 3.3 Linux

| Issue | รายละเอียด | วิธีแก้ |
|-------|-----------|---------|
| X11 vs Wayland | Clipboard mechanism ต่างกัน | arboard จัดการให้อัตโนมัติ |
| Wayland key sim | enigo มีปัญหาบน Wayland | ใช้ `wl-copy` + `wtype` fallback |
| Multiple clipboards | X11 มี PRIMARY + CLIPBOARD | ใช้ CLIPBOARD (arboard default) |

---

## 4. Clipboard Restore Strategy

### 4.1 ทำไมต้อง Restore

เพื่อไม่ทำให้ user สูญเสียเนื้อหาใน clipboard เดิม:

```rust
pub struct ClipboardGuard {
    original_text: Option<String>,
    clipboard: Clipboard,
}

impl ClipboardGuard {
    pub fn new() -> Result<Self> {
        let mut clipboard = Clipboard::new()?;
        let original_text = clipboard.get_text().ok();
        Ok(Self { original_text, clipboard })
    }

    pub fn set_and_paste(&mut self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        simulate_paste()?;
        Ok(())
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        // Restore original clipboard content
        if let Some(ref original) = self.original_text {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let _ = self.clipboard.set_text(original);
        }
    }
}
```

### 4.2 Limitations

- ไม่สามารถ restore clipboard ที่เป็นรูปภาพหรือ rich text ได้ (เฉพาะ plain text)
- ถ้า user paste เร็วมากหลัง inject อาจได้ข้อความผิด
- บาง app อาจ hold clipboard lock

---

## 5. Thai-Specific Validation

### 5.1 Unicode Verification

```rust
/// ตรวจสอบว่า text เป็น valid Thai Unicode
pub fn validate_thai_text(text: &str) -> bool {
    text.chars().all(|c| {
        c.is_ascii()                          // ASCII (English, numbers, punctuation)
        || ('\u{0E01}'..='\u{0E5B}').contains(&c)  // Thai block
        || c.is_whitespace()
        || c == '\n'
    })
}
```

### 5.2 Thai Character Ranges

| Range | Description |
|-------|-------------|
| U+0E01 - U+0E2E | Thai consonants (พยัญชนะ) |
| U+0E30 - U+0E3A | Thai vowels (สระ) |
| U+0E40 - U+0E45 | Thai leading vowels (สระนำ) |
| U+0E47 - U+0E4E | Thai tone marks (วรรณยุกต์) |
| U+0E50 - U+0E59 | Thai digits (ตัวเลขไทย) |

---

## 6. Append Mode (Phase 3)

สำหรับ long dictation — append แทน replace:

```rust
pub enum InjectMode {
    /// Replace: set clipboard + paste (default)
    Replace,
    /// Append: paste แล้วเพิ่มช่องว่างก่อน
    Append,
}

impl TextInjector {
    pub fn inject_with_mode(&self, text: &str, mode: InjectMode) -> Result<()> {
        match mode {
            InjectMode::Replace => self.inject(text),
            InjectMode::Append => {
                // เพิ่ม space/newline ก่อนข้อความ (ถ้าเป็นประโยคใหม่)
                let prefixed = format!(" {}", text);
                self.inject(&prefixed)
            }
        }
    }
}
```

---

## 7. Dependencies

```toml
[dependencies]
arboard = "3.4"   # Clipboard access (maintained by 1Password)
enigo = "0.3"     # Key simulation
```
