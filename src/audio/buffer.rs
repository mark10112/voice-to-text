//! Fixed-capacity circular (ring) buffer for `f32` audio samples.
//!
//! When the buffer is full, new samples **overwrite** the oldest data so that
//! the most-recent `capacity` samples are always available.  This matches
//! the push-to-talk scenario where we care about the tail of the recording,
//! not the head.
//!
//! # Example
//!
//! ```rust
//! use voice_to_text::audio::RingBuffer;
//!
//! let mut buf = RingBuffer::new(4);
//! buf.push_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]); // 5 items → capacity 4 → oldest dropped
//! let data = buf.drain();
//! assert_eq!(data, vec![2.0, 3.0, 4.0, 5.0]);
//! ```

// ---------------------------------------------------------------------------
// RingBuffer
// ---------------------------------------------------------------------------

/// A fixed-capacity circular buffer.
///
/// Generic over `T: Copy + Default` so it can store any `Copy` scalar, though
/// the audio pipeline uses `RingBuffer<f32>` exclusively.
///
/// ## Overflow behaviour
///
/// When [`push_slice`](Self::push_slice) would exceed `capacity`, the oldest
/// samples are silently overwritten.  The buffer never allocates beyond its
/// initial capacity.
pub struct RingBuffer<T> {
    buf: Vec<T>,
    capacity: usize,
    /// Index of the *next* write position (wraps around `capacity`).
    write_pos: usize,
    /// Number of valid samples currently stored (≤ `capacity`).
    len: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    /// Create a new ring buffer with the given `capacity`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity == 0`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be > 0");
        Self {
            buf: vec![T::default(); capacity],
            capacity,
            write_pos: 0,
            len: 0,
        }
    }

    /// Append `data` to the buffer.
    ///
    /// If the total number of samples exceeds `capacity`, the oldest samples
    /// are overwritten (circular behaviour).
    pub fn push_slice(&mut self, data: &[T]) {
        for &item in data {
            self.buf[self.write_pos] = item;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            if self.len < self.capacity {
                self.len += 1;
            }
        }
    }

    /// Drain all stored samples in chronological order and reset the buffer.
    ///
    /// After this call `len() == 0`.
    pub fn drain(&mut self) -> Vec<T> {
        if self.len == 0 {
            return Vec::new();
        }

        // When the buffer has never been fully filled, valid data starts at 0.
        // When the buffer is full (overflow has occurred), the oldest sample
        // sits at `write_pos` (the position the *next* write would go to).
        let read_pos = if self.len < self.capacity {
            0
        } else {
            self.write_pos
        };

        let mut result = Vec::with_capacity(self.len);
        for i in 0..self.len {
            result.push(self.buf[(read_pos + i) % self.capacity]);
        }

        self.clear();
        result
    }

    /// Discard all samples and reset the write position.
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.len = 0;
    }

    /// Number of valid samples currently stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the buffer contains no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Maximum number of samples the buffer can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` when the buffer has been filled to capacity at least
    /// once (i.e. overflow would occur on the next push).
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Current recording duration in seconds, assuming `sample_rate` Hz mono.
    pub fn duration_secs(&self, sample_rate: u32) -> f32 {
        if sample_rate == 0 {
            return 0.0;
        }
        self.len as f32 / sample_rate as f32
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Basic push / drain ------------------------------------------------

    #[test]
    fn push_and_drain_within_capacity() {
        let mut buf = RingBuffer::new(8);
        buf.push_slice(&[1.0_f32, 2.0, 3.0]);
        assert_eq!(buf.len(), 3);
        assert!(!buf.is_full());

        let data = buf.drain();
        assert_eq!(data, vec![1.0, 2.0, 3.0]);
        assert!(buf.is_empty());
    }

    #[test]
    fn push_exactly_capacity() {
        let mut buf = RingBuffer::new(4);
        buf.push_slice(&[1.0_f32, 2.0, 3.0, 4.0]);
        assert!(buf.is_full());

        let data = buf.drain();
        assert_eq!(data, vec![1.0, 2.0, 3.0, 4.0]);
    }

    // ---- Overflow (oldest sample discarded) --------------------------------

    #[test]
    fn overflow_by_one_drops_oldest() {
        let mut buf = RingBuffer::new(4);
        buf.push_slice(&[1.0_f32, 2.0, 3.0, 4.0, 5.0]); // 5 > capacity(4)

        assert_eq!(buf.len(), 4);
        let data = buf.drain();
        // 1.0 was overwritten; remaining order must be preserved
        assert_eq!(data, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn overflow_by_full_capacity_keeps_newest() {
        let mut buf = RingBuffer::new(4);
        // Push 8 items — only last 4 survive
        buf.push_slice(&[1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);

        assert_eq!(buf.len(), 4);
        let data = buf.drain();
        assert_eq!(data, vec![5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn multiple_overflows_in_separate_calls() {
        let mut buf = RingBuffer::new(3);
        buf.push_slice(&[1.0_f32, 2.0, 3.0]); // fill
        buf.push_slice(&[4.0, 5.0]); // 2 more → overwrites 1 and 2

        let data = buf.drain();
        assert_eq!(data, vec![3.0, 4.0, 5.0]);
    }

    // ---- Drain / clear semantics -------------------------------------------

    #[test]
    fn drain_clears_buffer() {
        let mut buf = RingBuffer::new(4);
        buf.push_slice(&[1.0_f32, 2.0]);
        let _ = buf.drain();

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn drain_empty_returns_empty_vec() {
        let mut buf: RingBuffer<f32> = RingBuffer::new(4);
        assert_eq!(buf.drain(), Vec::<f32>::new());
    }

    #[test]
    fn clear_resets_state() {
        let mut buf = RingBuffer::new(4);
        buf.push_slice(&[1.0_f32, 2.0, 3.0, 4.0, 5.0]);
        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        // Should be usable again after clear
        buf.push_slice(&[9.0_f32]);
        assert_eq!(buf.drain(), vec![9.0]);
    }

    #[test]
    fn reuse_after_drain() {
        let mut buf = RingBuffer::new(3);

        buf.push_slice(&[1.0_f32, 2.0, 3.0]);
        let first = buf.drain();
        assert_eq!(first, vec![1.0, 2.0, 3.0]);

        buf.push_slice(&[4.0_f32, 5.0]);
        let second = buf.drain();
        assert_eq!(second, vec![4.0, 5.0]);
    }

    // ---- Capacity / duration helpers ---------------------------------------

    #[test]
    fn capacity_reported_correctly() {
        let buf: RingBuffer<f32> = RingBuffer::new(1024);
        assert_eq!(buf.capacity(), 1024);
    }

    #[test]
    fn duration_secs_calculation() {
        let mut buf = RingBuffer::new(16_000);
        buf.push_slice(&vec![0.0_f32; 8_000]);
        // 8000 samples at 16kHz = 0.5 seconds
        assert!((buf.duration_secs(16_000) - 0.5).abs() < 1e-6);
    }

    // ---- Panic guard -------------------------------------------------------

    #[test]
    #[should_panic(expected = "RingBuffer capacity must be > 0")]
    fn zero_capacity_panics() {
        let _buf: RingBuffer<f32> = RingBuffer::new(0);
    }
}
