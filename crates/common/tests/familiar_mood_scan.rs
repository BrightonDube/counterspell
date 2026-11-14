//! Source scan for the mood rule (property P11).
//!
//! The mood must come only from measured facts. These files must not name
//! any AI, hint, guess, speech, prompt or preference identifier, even in a
//! comment. A match here means a new input path may have crept in.

const SCANNED: [(&str, &str); 2] = [
    ("facts.rs", include_str!("../src/familiar/facts.rs")),
    ("mood.rs", include_str!("../src/familiar/mood.rs")),
];

const FORBIDDEN: [&str; 6] = ["hint", "ai", "guess", "speech", "prompt", "prefs"];

/// Splits text into words at every character that is not an ASCII letter or digit.
/// Underscore also splits, so `rank_hint` is caught as the word `hint`.
fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
}

#[test]
fn mood_sources_name_no_ai_guess_speech_prompt_or_preference_identifier() {
    for (file, source) in SCANNED {
        for word in words(source) {
            assert!(
                !FORBIDDEN.contains(&word.as_str()),
                "{file} contains the forbidden word `{word}`"
            );
        }
    }
}

#[test]
fn the_scan_splits_on_underscore_so_compound_names_are_caught() {
    let found: Vec<String> = words("let rank_hint = 1;").collect();
    assert_eq!(found, vec!["let", "rank", "hint", "1"]);
}
