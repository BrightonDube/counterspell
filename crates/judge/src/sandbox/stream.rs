//! Bounded readers for what a sandboxed program writes. Both are pure state machines:
//! they take bytes and return results, with no I/O and no clock.
//!
//! - [`LineReader`] splits stdout into lines. A line longer than its cap is an overflow,
//!   and the call is killed (design section 9).
//! - [`StderrBuffer`] keeps the first 64 KiB of stderr, marks truncation, and keeps the
//!   last complete line for exception-class extraction. It never reports overflow (A2).
//!
//! Both give the same result however the bytes are split into chunks (property P5).

/// Most stored stderr bytes before truncation. The marker is added after them.
const STDERR_STORE_CAP: usize = 64 * 1024;
/// Most bytes kept of the last complete stderr line.
const STDERR_LINE_CAP: usize = 4 * 1024;
/// Appended to stored stderr once bytes have been dropped.
const TRUNCATION_MARKER: &[u8] = b"\n[stderr truncated]\n";

/// A line grew past the cap. The caller must kill the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

/// Splits bytes into lines, holding at most `cap` bytes of an unfinished line.
/// It also counts every byte it is given, so one call cannot exceed its byte budget.
#[derive(Debug)]
pub struct LineReader {
    cap: usize,
    budget_left: usize,
    pending: Vec<u8>,
    overflowed: bool,
}

impl LineReader {
    /// A reader that allows lines of up to `cap` bytes, not counting the newline, and
    /// has no total byte budget.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self::with_budget(cap, usize::MAX)
    }

    /// A reader with a line cap and a total byte budget for the call. Design section 9
    /// uses a 1 MiB budget for stdout.
    #[must_use]
    pub fn with_budget(cap: usize, budget: usize) -> Self {
        Self {
            cap,
            budget_left: budget,
            pending: Vec::new(),
            overflowed: false,
        }
    }

    /// Adds `chunk` and returns every line it completes, without the newline.
    ///
    /// The check runs before any byte is copied, so the reader never holds more than
    /// `cap` bytes. Once it has overflowed it keeps returning [`Overflow`].
    ///
    /// # Errors
    ///
    /// Returns [`Overflow`] when an unfinished line is longer than the cap, or when the
    /// call has received more bytes than its budget.
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<Vec<u8>>, Overflow> {
        if self.overflowed {
            return Err(Overflow);
        }
        if chunk.len() > self.budget_left {
            self.overflowed = true;
            return Err(Overflow);
        }
        self.budget_left -= chunk.len();
        let mut lines = Vec::new();
        let mut segments = chunk.split(|&byte| byte == b'\n').peekable();
        while let Some(segment) = segments.next() {
            if self.pending.len() + segment.len() > self.cap {
                self.overflowed = true;
                return Err(Overflow);
            }
            self.pending.extend_from_slice(segment);
            if segments.peek().is_some() {
                lines.push(std::mem::take(&mut self.pending));
            }
        }
        Ok(lines)
    }

    /// Starts a new call with a fresh byte budget. The unfinished line and any overflow
    /// carry over, so a line split across two calls is still one line.
    pub fn reset_budget(&mut self, budget: usize) {
        self.budget_left = budget;
    }

    /// How many bytes of the unfinished line are held right now.
    #[must_use]
    pub fn held(&self) -> usize {
        self.pending.len()
    }
}

/// Keeps a bounded copy of stderr. Bytes past the store cap are counted only as a
/// truncation, so a flood cannot grow memory.
#[derive(Debug, Default)]
pub struct StderrBuffer {
    stored: Vec<u8>,
    truncated: bool,
    current: Vec<u8>,
    last_line: Option<Vec<u8>>,
}

impl StderrBuffer {
    /// An empty buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `chunk`. Stores as much as fits under the cap and tracks the last line.
    pub fn push(&mut self, chunk: &[u8]) {
        let room = STDERR_STORE_CAP - self.stored.len();
        let take = room.min(chunk.len());
        self.stored.extend_from_slice(&chunk[..take]);
        if take < chunk.len() {
            self.truncated = true;
        }
        for &byte in chunk {
            self.track_line(byte);
        }
    }

    fn track_line(&mut self, byte: u8) {
        if byte == b'\n' {
            if !self.current.is_empty() {
                self.last_line = Some(std::mem::take(&mut self.current));
            }
        } else if self.current.len() < STDERR_LINE_CAP {
            self.current.push(byte);
        }
    }

    /// Whether any bytes were dropped after the store cap was reached.
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// The stored text, followed by the truncation marker if bytes were dropped.
    #[must_use]
    pub fn rendered(&self) -> Vec<u8> {
        let mut out = self.stored.clone();
        if self.truncated {
            out.extend_from_slice(TRUNCATION_MARKER);
        }
        out
    }

    /// Returns the stored text and clears the buffer for the next call.
    pub fn take_rendered(&mut self) -> Vec<u8> {
        let out = self.rendered();
        self.stored.clear();
        self.truncated = false;
        self.current.clear();
        self.last_line = None;
        out
    }

    /// The last non-empty complete line, capped at 4 KiB. An unterminated final line is not used.
    #[must_use]
    pub fn last_line(&self) -> Option<&[u8]> {
        self.last_line.as_deref()
    }
}
