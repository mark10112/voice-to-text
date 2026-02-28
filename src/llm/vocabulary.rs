//! User-defined vocabulary for STT correction.
//!
//! [`UserVocabulary`] persists a list of `(error, correction)` pairs as JSON
//! in the platform-appropriate config directory:
//!
//! | Platform | Path |
//! |----------|------|
//! | Windows  | `%APPDATA%\thai-vtt\user-vocab.json` |
//! | macOS    | `~/Library/Application Support/thai-vtt/user-vocab.json` |
//! | Linux    | `~/.config/thai-vtt/user-vocab.json` |

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// VocabEntry
// ---------------------------------------------------------------------------

/// A single user-defined correction entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabEntry {
    /// The mis-transcribed form (as produced by STT).
    pub error: String,
    /// The desired corrected form.
    pub correction: String,
    /// How many times this entry has been confirmed/used.
    pub frequency: u32,
}

// ---------------------------------------------------------------------------
// UserVocabulary
// ---------------------------------------------------------------------------

/// Manages user-defined STT correction entries.
///
/// Entries are persisted to JSON after every [`add`](UserVocabulary::add) call
/// so corrections survive app restarts.
pub struct UserVocabulary {
    entries: Vec<VocabEntry>,
    path: PathBuf,
}

impl UserVocabulary {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Load vocabulary from the platform config directory, or return an empty
    /// vocabulary when the file does not exist yet.
    pub fn load_or_default() -> Self {
        let path = Self::vocab_path();
        let entries = Self::load_entries(&path);
        Self { entries, path }
    }

    /// Load vocabulary from an explicit path (useful for tests).
    pub fn load_from(path: PathBuf) -> Self {
        let entries = Self::load_entries(&path);
        Self { entries, path }
    }

    fn load_entries(path: &PathBuf) -> Vec<VocabEntry> {
        if path.exists() {
            let data = std::fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    // -----------------------------------------------------------------------
    // Mutation
    // -----------------------------------------------------------------------

    /// Add or update a correction entry, then persist to disk.
    ///
    /// If an entry with the same `error` already exists its `correction` is
    /// updated and `frequency` incremented; otherwise a new entry is created.
    pub fn add(&mut self, error: String, correction: String) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.error == error) {
            entry.correction = correction;
            entry.frequency += 1;
        } else {
            self.entries.push(VocabEntry {
                error,
                correction,
                frequency: 1,
            });
        }
        self.save();
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the top-`n` most frequently used entries as `(error, correction)` pairs.
    pub fn top_entries(&self, n: usize) -> Vec<(String, String)> {
        let mut sorted = self.entries.clone();
        sorted.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        sorted
            .into_iter()
            .take(n)
            .map(|e| (e.error, e.correction))
            .collect()
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(&self.path, data);
        }
    }

    /// Platform-appropriate path for the vocabulary JSON file.
    fn vocab_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("thai-vtt")
            .join("user-vocab.json")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn vocab_in_temp() -> (UserVocabulary, tempfile::TempDir) {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("user-vocab.json");
        let vocab = UserVocabulary::load_from(path);
        (vocab, dir)
    }

    #[test]
    fn starts_empty() {
        let (vocab, _dir) = vocab_in_temp();
        assert!(vocab.is_empty());
        assert_eq!(vocab.len(), 0);
    }

    #[test]
    fn add_new_entry() {
        let (mut vocab, _dir) = vocab_in_temp();
        vocab.add("กาแฟ".into(), "กาแฟ".into());
        assert_eq!(vocab.len(), 1);
    }

    #[test]
    fn add_duplicate_increments_frequency() {
        let (mut vocab, _dir) = vocab_in_temp();
        vocab.add("ไก่".into(), "ไก่".into());
        vocab.add("ไก่".into(), "ไก่".into());
        assert_eq!(vocab.len(), 1);
        assert_eq!(vocab.entries[0].frequency, 2);
    }

    #[test]
    fn top_entries_sorted_by_frequency() {
        let (mut vocab, _dir) = vocab_in_temp();
        vocab.add("a".into(), "A".into());
        vocab.add("b".into(), "B".into());
        vocab.add("b".into(), "B".into());
        vocab.add("c".into(), "C".into());
        vocab.add("c".into(), "C".into());
        vocab.add("c".into(), "C".into());

        let top = vocab.top_entries(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "c"); // highest frequency first
        assert_eq!(top[1].0, "b");
    }

    #[test]
    fn top_entries_respects_n_limit() {
        let (mut vocab, _dir) = vocab_in_temp();
        for i in 0..10 {
            vocab.add(format!("error_{}", i), format!("fix_{}", i));
        }
        let top = vocab.top_entries(3);
        assert_eq!(top.len(), 3);
    }

    #[test]
    fn persists_and_reloads() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("user-vocab.json");

        {
            let mut vocab = UserVocabulary::load_from(path.clone());
            vocab.add("ผิด".into(), "ถูก".into());
        }

        let reloaded = UserVocabulary::load_from(path);
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded.entries[0].error, "ผิด");
        assert_eq!(reloaded.entries[0].correction, "ถูก");
    }
}
