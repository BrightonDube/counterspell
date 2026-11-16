//! Real-daemon protocol tests for the Node.js driver (language-drivers T04, protocol version 1).
//!
//! Ignored by default. They need `SANDBOX_DOCKER_SOCKET` pointing at the rootless sandbox
//! daemon, and the pinned `node:22-slim` image present there (the judge never pulls). No test
//! runs user code on the host.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_javascript_driver -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx_support;

use counterspell_judge::sandbox::backend::DriverChannel;
use sbx_support::{ask, finish, int, list_of, signature, start, stderr_until_exit, Mode, Reply};
use serde_json::{json, Value};

const JAVASCRIPT: &str = "javascript";

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_callable_module_export_answers() {
    let solution = "module.exports = function add(a, b) { return a + b; };\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([2, 3])).await,
        Reply::Answer(json!(5))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_named_export_is_checked_first() {
    let solution = "module.exports = { add: (a, b) => a + b, other: () => 0 };\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([4, 5])).await,
        Reply::Answer(json!(9))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_list_and_nested_list_answers() {
    let solution = "module.exports = { pairs: (xs) => xs.map((x) => [x, x * 2]) };\n";
    let sig = signature(
        "pairs",
        &[("xs", list_of(&int()))],
        &list_of(&list_of(&int())),
    );
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

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
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_boolean_answer() {
    let solution = "module.exports = { isEven: (n) => n % 2 === 0 };\n";
    let bool_sig = signature("isEven", &[("n", int())], &json!({"kind": "bool"}));
    let (session, mut driver) = start(JAVASCRIPT, solution, bool_sig, Mode::Normal).await;

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
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_cyclic_return_is_refused() {
    let solution = "module.exports = function f(n) { const a = [n]; a.push(a); return a; };\n";
    let sig = signature("f", &[("n", int())], &list_of(&int()));
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_string_answer() {
    let solution = "module.exports = { shout: (s) => s.toUpperCase() };\n";
    let sig = signature(
        "shout",
        &[("s", json!({"kind": "string"}))],
        &json!({"kind": "string"}),
    );
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!(["abc"])).await,
        Reply::Answer(json!("ABC"))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_null_answer_where_allowed() {
    let solution = "module.exports = { firstOrNull: (xs) => (xs.length ? xs[0] : null) };\n";
    let nullable_int = json!({"kind": "int", "nullable": true});
    let sig = signature("firstOrNull", &[("xs", list_of(&int()))], &nullable_int);
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

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
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_repeated_calls_keep_their_ids() {
    let solution = "module.exports = function double(n) { return 2 * n; };\n";
    let sig = signature("double", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

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
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_prints_are_captured_and_cannot_fake_an_answer() {
    let solution = "module.exports = function f(n) {\n  console.log('hello');\n  console.log('{\"t\": 0, \"out\": \"accepted\"}');\n  return n;\n};\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([9])).await,
        Reply::Answer(json!(9))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_awaits_a_promise_result() {
    let solution = "module.exports = { f: async (n) => n + 1 };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([1])).await,
        Reply::Answer(json!(2))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_rejected_promise_exits_without_an_answer() {
    let solution = "module.exports = { f: async () => { throw new Error('boom'); } };\n";
    let sig = signature("f", &[], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_raising_function_exits_without_an_answer() {
    let solution = "module.exports = function f(n) { throw new Error('boom'); };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_undefined_return_is_refused() {
    let solution = "module.exports = function f(n) { };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_boolean_where_int_expected_is_refused() {
    let solution = "module.exports = function f(n) { return true; };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_integer_outside_json_safe_range_is_refused() {
    let solution = "module.exports = function f(n) { return 2 ** 60; };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_non_finite_float_is_refused() {
    let solution = "module.exports = function f(n) { return NaN; };\n";
    let sig = signature("f", &[("n", int())], &json!({"kind": "float"}));
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_wrong_argument_count_is_refused() {
    let solution = "module.exports = function f(n) { return n; };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1, 2])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_blank_line_is_refused() {
    let solution = "module.exports = function f(n) { return n; };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    driver.write_line(b"\n").await.expect("the line is written");
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(
        !stderr.is_empty(),
        "the driver explains the refusal on stderr"
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_missing_function_fails_at_load() {
    let solution = "module.exports = { somethingElse: (n) => n };\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Normal).await;

    // The driver exits while it loads the solution, so no call is sent: read its diagnostic.
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(stderr.contains("does not define f"), "stderr was: {stderr}");

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_bench_mode_answers_like_normal_mode() {
    let solution = "module.exports = function add(a, b) { return a + b; };\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(JAVASCRIPT, solution, sig, Mode::Bench).await;

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
