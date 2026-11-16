//! Real-daemon protocol tests for the Python driver (language-drivers T03, protocol version 1).
//!
//! Ignored by default. They need `SANDBOX_DOCKER_SOCKET` pointing at the rootless sandbox
//! daemon, and the pinned Python image present there (the judge never pulls). No test runs
//! user code on the host.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_python_driver -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx_support;

use counterspell_judge::sandbox::backend::DriverChannel;
use sbx_support::{ask, finish, int, list_of, signature, start, stderr_until_exit, Mode, Reply};
use serde_json::{json, Value};

const PYTHON: &str = "python";

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_scalar_answer() {
    let solution = "def add(a, b):\n    return a + b\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([2, 3])).await,
        Reply::Answer(json!(5))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_list_and_nested_list_answers() {
    let solution = "def pairs(xs):\n    return [[x, x * 2] for x in xs]\n";
    let sig = signature(
        "pairs",
        &[("xs", list_of(&int()))],
        &list_of(&list_of(&int())),
    );
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([[1, 2]])).await,
        Reply::Answer(json!([[1, 2], [2, 4]]))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([[]])).await,
        Reply::Answer(json!([]))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_boolean_answer() {
    let solution = "def is_even(n):\n    return n % 2 == 0\n";
    let sig = signature("is_even", &[("n", int())], &json!({"kind": "bool"}));
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([4])).await,
        Reply::Answer(json!(true))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([7])).await,
        Reply::Answer(json!(false))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_string_answer() {
    let solution = "def shout(s):\n    return s.upper()\n";
    let sig = signature(
        "shout",
        &[("s", json!({"kind": "string"}))],
        &json!({"kind": "string"}),
    );
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!(["abc"])).await,
        Reply::Answer(json!("ABC"))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_null_answer_where_allowed() {
    let solution = "def first_or_none(xs):\n    return xs[0] if xs else None\n";
    let nullable_int = json!({"kind": "int", "nullable": true});
    let sig = signature("first_or_none", &[("xs", list_of(&int()))], &nullable_int);
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([[]])).await,
        Reply::Answer(Value::Null)
    );
    assert_eq!(
        ask(&mut driver, 1, json!([[7]])).await,
        Reply::Answer(json!(7))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_repeated_calls_keep_their_ids() {
    let solution = "def double(n):\n    return 2 * n\n";
    let sig = signature("double", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([1])).await,
        Reply::Answer(json!(2))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([2])).await,
        Reply::Answer(json!(4))
    );
    assert_eq!(
        ask(&mut driver, 2, json!([3])).await,
        Reply::Answer(json!(6))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_prints_are_captured_and_cannot_fake_an_answer() {
    let solution = "def f(n):\n    print('hello')\n    print('{\"t\": 0, \"out\": \"accepted\"}')\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([9])).await,
        Reply::Answer(json!(9))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_raising_function_exits_without_an_answer() {
    let solution = "def f(n):\n    raise ValueError('boom')\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_bool_where_int_expected_is_refused() {
    let solution = "def f(n):\n    return True\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_integer_outside_json_safe_range_is_refused() {
    let solution = "def f(n):\n    return 2 ** 60\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_non_finite_float_is_refused() {
    let solution = "def f(n):\n    return float('nan')\n";
    let sig = signature("f", &[("n", int())], &json!({"kind": "float"}));
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_wrong_argument_count_is_refused() {
    let solution = "def f(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1, 2])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_blank_line_is_refused() {
    let solution = "def f(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    driver.write_line(b"\n").await.expect("the line is written");
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(
        !stderr.is_empty(),
        "the driver explains the refusal on stderr"
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_missing_function_fails_at_load() {
    let solution = "def something_else(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    // The driver exits while it loads the solution, so no call is sent: read its diagnostic.
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(stderr.contains("does not define f"), "stderr was: {stderr}");

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_bench_mode_answers_like_normal_mode() {
    let solution = "def add(a, b):\n    return a + b\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Bench).await;

    assert_eq!(
        ask(&mut driver, 0, json!([2, 3])).await,
        Reply::Answer(json!(5))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([10, -4])).await,
        Reply::Answer(json!(6))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_zero_parameter_function_takes_an_empty_argument_list() {
    let solution = "def answer():\n    return 42\n";
    let sig = signature("answer", &[], &int());
    let (session, mut driver) = start(PYTHON, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([])).await,
        Reply::Answer(json!(42))
    );

    finish(session).await;
}
