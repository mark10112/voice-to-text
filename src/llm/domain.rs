//! Keyword-based domain detection for Thai (and mixed) text.
//!
//! [`DomainDetector`] scans a text window for domain-specific keywords and
//! returns the best-matching domain label (`"Medical"`, `"Legal"`,
//! `"Technical"`, `"Casual"`).  A minimum keyword count threshold prevents
//! false positives.

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

struct DomainConfig {
    name: &'static str,
    keywords: &'static [&'static str],
    /// Minimum number of keyword matches required to trigger this domain.
    threshold: usize,
}

// ---------------------------------------------------------------------------
// Static domain definitions
// ---------------------------------------------------------------------------

static DOMAINS: &[DomainConfig] = &[
    DomainConfig {
        name: "Medical",
        keywords: &[
            "ผู้ป่วย",
            "ยา",
            "อาการ",
            "โรค",
            "แพทย์",
            "วินิจฉัย",
            "โรงพยาบาล",
            "เบาหวาน",
            "ความดัน",
            "การรักษา",
        ],
        threshold: 2,
    },
    DomainConfig {
        name: "Legal",
        keywords: &[
            "กฎหมาย",
            "สัญญา",
            "ศาล",
            "จำเลย",
            "โจทก์",
            "คดี",
            "ข้อพิพาท",
            "พระราชบัญญัติ",
            "ทนายความ",
            "คำพิพากษา",
        ],
        threshold: 2,
    },
    DomainConfig {
        name: "Technical",
        keywords: &[
            "code",
            "function",
            "server",
            "deploy",
            "database",
            "API",
            "bug",
            "ซอฟต์แวร์",
            "ระบบ",
            "โปรแกรม",
            "คอมพิวเตอร์",
        ],
        threshold: 2,
    },
    DomainConfig {
        name: "Casual",
        keywords: &[
            "คุย",
            "เล่า",
            "เพื่อน",
            "กิน",
            "ไปเที่ยว",
            "สนุก",
            "หัวเราะ",
            "นัด",
        ],
        threshold: 3,
    },
];

// ---------------------------------------------------------------------------
// DomainDetector
// ---------------------------------------------------------------------------

/// Detects the likely domain of a text excerpt based on keyword frequency.
///
/// Domains checked (in order of priority): Medical → Legal → Technical → Casual.
/// If multiple domains meet their threshold the one with the most keyword
/// matches wins.  Returns `None` when no domain reaches its threshold.
///
/// # Example
/// ```rust
/// use voice_to_text::llm::DomainDetector;
///
/// let detector = DomainDetector::new();
/// let domain = detector.detect("ผู้ป่วยมีความดันสูง วินิจฉัยโดยแพทย์");
/// assert_eq!(domain, Some("Medical".to_string()));
/// ```
pub struct DomainDetector;

impl DomainDetector {
    /// Create a new detector with the built-in domain definitions.
    pub fn new() -> Self {
        Self
    }

    /// Detect the most likely domain label for `text`, or `None`.
    pub fn detect(&self, text: &str) -> Option<String> {
        DOMAINS
            .iter()
            .filter_map(|domain| {
                let count = domain
                    .keywords
                    .iter()
                    .filter(|kw| text.contains(**kw))
                    .count();
                if count >= domain.threshold {
                    Some((domain.name, count))
                } else {
                    None
                }
            })
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name.to_string())
    }
}

impl Default for DomainDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_medical_domain() {
        let d = DomainDetector::new();
        let text = "ผู้ป่วยมีความดันสูง วินิจฉัยโดยแพทย์ที่โรงพยาบาล";
        assert_eq!(d.detect(text), Some("Medical".to_string()));
    }

    #[test]
    fn detects_legal_domain() {
        let d = DomainDetector::new();
        let text = "คดีนี้ขึ้นศาล โจทก์ยื่นฟ้องตามกฎหมาย";
        assert_eq!(d.detect(text), Some("Legal".to_string()));
    }

    #[test]
    fn detects_technical_domain() {
        let d = DomainDetector::new();
        let text = "deploy server แล้ว ยัง bug อยู่ใน function นี้";
        assert_eq!(d.detect(text), Some("Technical".to_string()));
    }

    #[test]
    fn returns_none_below_threshold() {
        let d = DomainDetector::new();
        // Only one medical keyword — below threshold of 2
        let text = "ผู้ป่วยมาหา";
        assert_eq!(d.detect(text), None);
    }

    #[test]
    fn returns_none_for_unrelated_text() {
        let d = DomainDetector::new();
        assert_eq!(d.detect("สวัสดีครับ ขอบคุณมากครับ"), None);
    }

    #[test]
    fn prefers_higher_match_count() {
        let d = DomainDetector::new();
        // Medical has 4 matches, Legal has 2 — Medical should win
        let text = "ผู้ป่วยมีโรคเบาหวาน แพทย์วินิจฉัยว่ามีอาการ และต้องทำสัญญา ปรึกษากฎหมาย";
        assert_eq!(d.detect(text), Some("Medical".to_string()));
    }
}
