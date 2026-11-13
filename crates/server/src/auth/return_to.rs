//! Sanitising the `return_to` path that a person asks to go back to after signing in.
//!
//! Only a path on this site is allowed. Anything else becomes `/`, so a crafted link cannot send
//! someone to another website.

use percent_encoding::percent_decode_str;

/// The longest `return_to` value accepted, in characters.
pub const MAX_RETURN_TO_CHARS: usize = 200;

/// The fallback destination when the requested one is missing or not acceptable.
pub const DEFAULT_RETURN_TO: &str = "/";

/// Returns a safe path to redirect to. The result is always a path that starts with one `/`.
#[must_use]
pub fn sanitize_return_to(input: Option<&str>) -> String {
    match input {
        Some(raw)
            if is_acceptable(raw)
                && is_acceptable(&percent_decode_str(raw).decode_utf8_lossy()) =>
        {
            raw.to_owned()
        }
        _ => DEFAULT_RETURN_TO.to_owned(),
    }
}

/// One path must start with a single `/`, stay short, and contain no control characters.
fn is_acceptable(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.starts_with("/\\")
        && path.chars().count() <= MAX_RETURN_TO_CHARS
        && !path.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use url::Url;

    const SITE: &str = "https://counterspell.example";

    fn stays_on_site(path: &str) -> bool {
        let base = Url::parse(SITE).expect("base");
        base.join(path)
            .is_ok_and(|joined| joined.origin() == base.origin())
    }

    #[test]
    fn a_normal_path_is_kept() {
        assert_eq!(
            sanitize_return_to(Some("/crypts/arrays?x=1")),
            "/crypts/arrays?x=1"
        );
        assert_eq!(sanitize_return_to(Some("/ok?x=%20")), "/ok?x=%20");
    }

    #[test]
    fn missing_or_hostile_values_fall_back_to_root() {
        for input in [
            None,
            Some(""),
            Some("//evil.com"),
            Some("/\\evil.com"),
            Some("/%2f%2fevil.com"),
            Some("/%5cevil.com"),
            Some("https://evil.example"),
            Some("javascript:alert(1)"),
            Some("relative/path"),
            Some("/a\u{0007}b"),
            Some("/%0a"),
        ] {
            assert_eq!(sanitize_return_to(input), "/", "{input:?}");
        }
    }

    #[test]
    fn a_value_of_200_characters_is_kept_and_201_is_refused() {
        let at_limit = format!("/{}", "a".repeat(199));
        assert_eq!(sanitize_return_to(Some(&at_limit)), at_limit);
        let over = format!("/{}", "a".repeat(200));
        assert_eq!(sanitize_return_to(Some(&over)), "/");
    }

    proptest! {
        #[test]
        fn the_result_always_stays_on_the_site(input in "\\PC{0,260}") {
            let result = sanitize_return_to(Some(&input));
            prop_assert!(result.starts_with('/'));
            prop_assert!(stays_on_site(&result));
        }

        #[test]
        fn encoded_slashes_never_escape_the_site(input in "(/|%2f|%2F|%5c|%5C|\\\\|[a-z.]){0,12}") {
            let result = sanitize_return_to(Some(&input));
            prop_assert!(stays_on_site(&result), "{result}");
        }
    }
}
