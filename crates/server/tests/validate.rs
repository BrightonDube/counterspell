//! Submission checks (VAL-1 size, VAL-2 parse, VAL-3 plain text). They run on the server before a run is
//! queued. Browser checks are only for feedback, and the sandbox is the real security boundary.

use axum::http::StatusCode;
use counterspell_server::{
    error::ApiError,
    validate::{decode_source, validate_source, ValidationError},
};
use proptest::prelude::*;

const LIMIT: u64 = 65_536;

#[test]
fn accepts_valid_python() {
    validate_source("python", "def add(a, b):\n    return a + b\n", LIMIT).expect("valid python");
}

#[test]
fn accepts_valid_javascript() {
    validate_source(
        "javascript",
        "function add(a, b) {\n  return a + b;\n}\n",
        LIMIT,
    )
    .expect("valid javascript");
}

#[test]
fn rejects_python_with_a_missing_colon_on_line_one() {
    let error = validate_source("python", "def add(a, b)\n    return a + b\n", LIMIT)
        .expect_err("missing colon");

    assert!(
        matches!(error, ValidationError::DoesNotParse { line: 1, .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_javascript_with_a_dangling_operator() {
    let error =
        validate_source("javascript", "let total = 1 +;\n", LIMIT).expect_err("syntax error");

    assert!(
        matches!(error, ValidationError::DoesNotParse { line: 1, .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_a_pasted_shell_script_as_python() {
    // `rm -rf /tmp/work` is a valid Python expression, so the shell loop below is used instead.
    let script = "#!/bin/sh\nfor f in *.log; do\n  rm \"$f\"\ndone\n";
    let error = validate_source("python", script, LIMIT).expect_err("shell is not python");

    assert!(
        matches!(error, ValidationError::DoesNotParse { line: 2, .. }),
        "{error:?}"
    );
}

#[test]
fn accepts_source_exactly_at_the_limit() {
    let mut code = "x = 1\n".to_owned();
    code.push_str(&"#".repeat(usize::try_from(LIMIT).unwrap_or(0) - 7));
    code.push('\n');
    assert_eq!(code.len() as u64, LIMIT);

    validate_source("python", &code, LIMIT).expect("exactly at the limit is allowed");
}

#[test]
fn rejects_one_byte_over_the_limit_and_states_both_sizes() {
    let mut code = "x = 1\n".to_owned();
    code.push_str(&"#".repeat(usize::try_from(LIMIT).unwrap_or(0) - 6));
    code.push('\n');
    let actual = code.len() as u64;

    let error = validate_source("python", &code, LIMIT).expect_err("one byte too many");

    assert_eq!(
        error,
        ValidationError::TooLarge {
            limit: LIMIT,
            actual
        }
    );
    let message = error.to_string();
    assert!(
        message.contains("65536") && message.contains("65537"),
        "{message}"
    );
}

#[test]
fn size_is_counted_in_bytes_not_characters() {
    // 40 000 two-byte characters: 40 002 characters, but 80 002 bytes, which is over the limit.
    let code = format!("#{}\n", "é".repeat(40_000));

    let error = validate_source("python", &code, LIMIT).expect_err("bytes exceed the limit");

    assert!(
        matches!(error, ValidationError::TooLarge { .. }),
        "{error:?}"
    );
}

#[test]
fn rejects_a_nul_byte() {
    let error = validate_source("python", "x = 1\u{0}\n", LIMIT).expect_err("NUL is not allowed");

    assert!(
        matches!(
            error,
            ValidationError::ForbiddenCharacter { line: 1, column: 6 }
        ),
        "{error:?}"
    );
}

#[test]
fn rejects_an_escape_control_character_without_echoing_it() {
    let error =
        validate_source("python", "x = 1\u{1b}[31m\n", LIMIT).expect_err("ESC is not allowed");

    assert!(
        matches!(error, ValidationError::ForbiddenCharacter { .. }),
        "{error:?}"
    );
    assert!(!error.to_string().contains('\u{1b}'));
}

#[test]
fn accepts_tabs_and_both_kinds_of_line_ending() {
    validate_source("python", "def f():\n\treturn 1\r\n", LIMIT).expect("tab and CRLF are allowed");
    validate_source(
        "javascript",
        "function f() {\r\n\treturn 1;\r\n}\r\n",
        LIMIT,
    )
    .expect("tab and CRLF are allowed");
}

#[test]
fn rejects_an_unsupported_language() {
    let error =
        validate_source("java", "class A {}\n", LIMIT).expect_err("java is a stretch language");

    assert_eq!(error, ValidationError::UnsupportedLanguage);
}

#[test]
fn decode_accepts_utf8_and_refuses_other_bytes() {
    assert_eq!(
        decode_source("x = 'é'\n".as_bytes()).expect("utf-8"),
        "x = 'é'\n"
    );
    assert_eq!(
        decode_source(&[0xff, 0xfe, b'x']),
        Err(ValidationError::NotUtf8)
    );
}

#[test]
fn deeply_nested_expression_neither_panics_nor_hangs() {
    let code = format!("x = {}1{}\n", "(".repeat(3_000), ")".repeat(3_000));

    let result = validate_source("python", &code, LIMIT);

    assert!(
        matches!(result, Ok(()) | Err(ValidationError::DoesNotParse { .. })),
        "{result:?}"
    );
}

#[test]
fn validation_errors_map_to_the_design_status_codes() {
    let too_large = ApiError::from(ValidationError::TooLarge {
        limit: 1,
        actual: 2,
    });
    let control = ApiError::from(ValidationError::ForbiddenCharacter { line: 1, column: 1 });
    let parse = ApiError::from(ValidationError::DoesNotParse {
        language: "python".to_owned(),
        line: 1,
        column: 1,
    });

    assert_eq!(too_large.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(control.status(), StatusCode::BAD_REQUEST);
    assert_eq!(parse.status(), StatusCode::BAD_REQUEST);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Any byte sequence, decoded or not, must never make validation panic.
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        if let Ok(text) = decode_source(&bytes) {
            let _ = validate_source("python", text, LIMIT);
            let _ = validate_source("javascript", text, LIMIT);
        }
    }

    /// A NUL byte anywhere in a short source is always refused as a forbidden character.
    #[test]
    fn a_nul_anywhere_is_always_refused(prefix in "[ -~\n]{0,200}", suffix in "[ -~\n]{0,200}") {
        let code = format!("{prefix}\u{0}{suffix}");
        let result = validate_source("python", &code, LIMIT);
        prop_assert!(matches!(result, Err(ValidationError::ForbiddenCharacter { .. })), "{:?}", result);
    }
}
