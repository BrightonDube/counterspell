//! Display sanitiser for program output (requirement VAL-5). Output is shown as plain
//! text, and this module makes sure no control character or terminal escape reaches
//! the screen, whatever the program printed.

/// Most bytes stored per output snippet.
pub const STORED_CAP: usize = 64 * 1024;
/// Most bytes kept per detail field in a public event.
pub const DETAIL_CAP: usize = 4 * 1024;

const REPLACEMENT: char = '\u{FFFD}';

/// Sanitised text and whether it was cut to fit the cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sanitized {
    /// Valid UTF-8 with no forbidden control characters. Never longer than the cap.
    pub text: String,
    /// True exactly when the sanitised output was longer than the cap.
    pub truncated: bool,
}

/// Turns program output into text that is safe to show.
///
/// Invalid UTF-8 becomes U+FFFD. NUL is removed. Every other control character except
/// tab and newline, and DEL, becomes U+FFFD, which also neutralises terminal escapes.
/// The result is cut to at most `cap` bytes, on a character boundary.
#[must_use]
pub fn sanitize_display(bytes: &[u8], cap: usize) -> Sanitized {
    let decoded = String::from_utf8_lossy(bytes);
    let mut clean = String::new();
    let mut over_cap = false;

    for ch in decoded.chars() {
        match ch {
            '\0' => {}
            '\t' | '\n' => clean.push(ch),
            c if is_forbidden_control(c) => clean.push(REPLACEMENT),
            c => clean.push(c),
        }
        if clean.len() > cap {
            over_cap = true;
            break;
        }
    }

    if !over_cap {
        return Sanitized {
            text: clean,
            truncated: false,
        };
    }
    let mut end = cap;
    while !clean.is_char_boundary(end) {
        end -= 1;
    }
    clean.truncate(end);
    Sanitized {
        text: clean,
        truncated: true,
    }
}

/// C0 controls other than tab and newline, plus DEL. NUL is handled separately.
fn is_forbidden_control(c: char) -> bool {
    (c as u32) < 0x20 && c != '\t' && c != '\n' || c == '\u{7f}'
}
