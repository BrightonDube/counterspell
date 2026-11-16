//! Real-daemon parity test for the Python and Node.js drivers (language-drivers T06).
//!
//! Each case is the same function written in both languages, with the same signature and the
//! same calls. Both drivers run in the rootless sandbox, in normal and in bench mode, and their
//! replies must match: the same parsed answers, and the same exit code when a call fails.
//!
//! Ignored by default. Needs `SANDBOX_DOCKER_SOCKET`, the pinned Python and Node images on the
//! rootless daemon, and never runs on the host.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_driver_parity -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx_support;

use counterspell_judge::sandbox::backend::DriverChannel;
use sbx_support::{ask, finish, int, list_of, signature, start, stderr_until_exit, Mode, Reply};
use serde_json::{json, Value};

/// One parity case: the same function in both languages, the signature, and the calls.
struct Case {
    name: &'static str,
    python: &'static str,
    javascript: &'static str,
    signature: Vec<u8>,
    calls: Vec<Value>,
}

fn cases() -> Vec<Case> {
    let nullable_int = json!({"kind": "int", "nullable": true});
    vec![
        Case {
            name: "scalar",
            python: "def add(a, b):\n    return a + b\n",
            javascript: "module.exports = function add(a, b) { return a + b; };\n",
            signature: signature("add", &[("a", int()), ("b", int())], &int()),
            calls: vec![json!([2, 3]), json!([10, -4]), json!([0, 0])],
        },
        Case {
            name: "nested list",
            python: "def pairs(xs):\n    return [[x, x * 2] for x in xs]\n",
            javascript: "module.exports = function pairs(xs) { return xs.map((x) => [x, x * 2]); };\n",
            signature: signature("pairs", &[("xs", list_of(&int()))], &list_of(&list_of(&int()))),
            calls: vec![json!([[1, 2]]), json!([[]])],
        },
        Case {
            name: "boolean",
            python: "def is_even(n):\n    return n % 2 == 0\n",
            javascript: "module.exports = function is_even(n) { return n % 2 === 0; };\n",
            signature: signature("is_even", &[("n", int())], &json!({"kind": "bool"})),
            calls: vec![json!([4]), json!([7])],
        },
        Case {
            name: "null where allowed",
            python: "def first_or_none(xs):\n    return xs[0] if xs else None\n",
            javascript: "module.exports = function first_or_none(xs) { return xs.length ? xs[0] : null; };\n",
            signature: signature("first_or_none", &[("xs", list_of(&int()))], &nullable_int),
            calls: vec![json!([[]]), json!([[7]])],
        },
        Case {
            name: "user prints",
            python: "def f(n):\n    print('hello')\n    print('{\"t\": 0, \"out\": \"accepted\"}')\n    return n\n",
            javascript: "module.exports = function f(n) {\n  console.log('hello');\n  console.log('{\"t\": 0, \"out\": \"accepted\"}');\n  return n;\n};\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([9])],
        },
        Case {
            name: "raising function",
            python: "def f(n):\n    raise ValueError('boom')\n",
            javascript: "module.exports = function f(n) { throw new Error('boom'); };\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([1])],
        },
        Case {
            name: "boolean where int expected",
            python: "def f(n):\n    return True\n",
            javascript: "module.exports = function f(n) { return true; };\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([1])],
        },
        Case {
            name: "wrong argument count",
            python: "def f(n):\n    return n\n",
            javascript: "module.exports = function f(n) { return n; };\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([1, 2])],
        },
    ]
}

/// Runs every call of a case in one container and returns the reply to each call. A driver that
/// has ended stops the list, because it cannot answer further calls.
async fn replies(
    lang: &str,
    solution: &str,
    sig: &[u8],
    calls: &[Value],
    mode: Mode,
) -> Vec<Reply> {
    let (session, mut driver) = start(lang, solution, sig.to_vec(), mode).await;
    let mut out = Vec::new();
    for (id, args) in calls.iter().enumerate() {
        let reply = ask(&mut driver, id as u64, args.clone()).await;
        let ended = matches!(reply, Reply::Exited(_));
        out.push(reply);
        if ended {
            break;
        }
    }
    finish(session).await;
    out
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python and node images on the rootless daemon"]
async fn sbx_python_and_javascript_drivers_agree_on_every_case() {
    let mut disagreements = Vec::new();
    for case in cases() {
        for mode in [Mode::Normal, Mode::Bench] {
            let python = replies("python", case.python, &case.signature, &case.calls, mode).await;
            let javascript = replies(
                "javascript",
                case.javascript,
                &case.signature,
                &case.calls,
                mode,
            )
            .await;
            if python != javascript {
                disagreements.push(format!(
                    "{} ({mode:?}): python {python:?} vs javascript {javascript:?}",
                    case.name
                ));
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "the drivers disagree:\n{}",
        disagreements.join("\n")
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python and node images on the rootless daemon"]
async fn sbx_driver_stderr_carries_only_diagnostics() {
    // A failing call writes its explanation to stderr and never to the answer stream.
    let sig = signature("f", &[("n", int())], &int());
    let solution = "module.exports = function f(n) { throw new Error('boom'); };\n";
    let (session, mut driver) = start("javascript", solution, sig, Mode::Normal).await;

    driver
        .write_line(b"{\"t\":0,\"args\":[1]}\n")
        .await
        .expect("the driver accepts a call line");
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(
        stderr.contains("boom"),
        "the explanation is on stderr: {stderr}"
    );

    finish(session).await;
}
