//! Model tests for the display sanitiser (requirements R13.3, R13.4, R14.5, property
//! P11). No Docker, no I/O.

use counterspell_judge::sandbox::sanitize::{sanitize_display, DETAIL_CAP, STORED_CAP};
use proptest::prelude::*;

const REPLACEMENT: char = '\u{FFFD}';

fn has_forbidden_control(text: &str) -> bool {
    text.chars()
        .any(|c| c == '\0' || (c != '\t' && c != '\n' && ((c as u32) < 0x20 || c == '\u{7f}')))
}

#[test]
fn text_within_the_cap_passes_through_unchanged() {
    let out = sanitize_display(b"hello\tworld\n", 64);
    assert_eq!(out.text, "hello\tworld\n");
    assert!(!out.truncated);
}

#[test]
fn nul_bytes_are_removed() {
    let out = sanitize_display(b"a\0b", 64);
    assert_eq!(out.text, "ab");
}

#[test]
fn an_ansi_escape_is_neutralised_not_passed_through() {
    // ESC (0x1B) becomes the replacement character, so no terminal sequence survives.
    let out = sanitize_display(b"\x1b[31mred\x1b[0m", 64);
    assert_eq!(out.text, format!("{REPLACEMENT}[31mred{REPLACEMENT}[0m"));
    assert!(!out.text.contains('\u{1b}'));
}

#[test]
fn other_c0_controls_and_delete_become_the_replacement_character() {
    let out = sanitize_display(b"\x07bell\r\x7f", 64);
    assert_eq!(
        out.text,
        format!("{REPLACEMENT}bell{REPLACEMENT}{REPLACEMENT}")
    );
}

#[test]
fn invalid_utf8_becomes_the_replacement_character() {
    let out = sanitize_display(b"ok\xff!", 64);
    assert_eq!(out.text, format!("ok{REPLACEMENT}!"));
}

#[test]
fn a_multibyte_character_is_never_split_at_the_cap() {
    // "é" is two bytes. A cap of 3 bytes must keep "aé" and drop the rest.
    let out = sanitize_display("aéé".as_bytes(), 3);
    assert_eq!(out.text, "aé");
    assert!(out.truncated);
    assert!(out.text.len() <= 3);
}

#[test]
fn text_exactly_at_the_cap_is_not_truncated() {
    let out = sanitize_display(b"abcd", 4);
    assert_eq!(out.text, "abcd");
    assert!(!out.truncated);
}

#[test]
fn text_one_byte_over_the_cap_is_truncated() {
    let out = sanitize_display(b"abcde", 4);
    assert_eq!(out.text, "abcd");
    assert!(out.truncated);
}

#[test]
fn the_documented_caps_are_64_kib_and_4_kib() {
    assert_eq!(STORED_CAP, 64 * 1024);
    assert_eq!(DETAIL_CAP, 4 * 1024);
}

proptest! {
    /// P11: output is valid text, within the cap, free of forbidden controls, stable
    /// when sanitised again, and marked truncated exactly when the full output is longer.
    #[test]
    fn output_is_bounded_clean_idempotent_and_flagged_exactly(
        bytes in prop::collection::vec(any::<u8>(), 0..400),
        cap in 0usize..=200,
    ) {
        let out = sanitize_display(&bytes, cap);
        prop_assert!(out.text.len() <= cap);
        prop_assert!(!has_forbidden_control(&out.text));

        let full = sanitize_display(&bytes, usize::MAX);
        prop_assert_eq!(out.truncated, full.text.len() > cap);
        prop_assert!(full.text.starts_with(&out.text));

        let again = sanitize_display(out.text.as_bytes(), cap);
        prop_assert_eq!(again.text, out.text);
        prop_assert!(!again.truncated);
    }
}
