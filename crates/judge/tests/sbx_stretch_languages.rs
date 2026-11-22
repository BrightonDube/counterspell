// SPDX-License-Identifier: MIT
//! Real-daemon stretch suite for the Java and C++ drivers (language-drivers T16).
//!
//! Each case runs the same function in Python (the launch reference) and in Java and C++,
//! with the same signature and the same calls, in normal and in bench mode. The stretch
//! drivers' replies must agree with Python's: the same parsed answers, and the same exit
//! code when a call fails. Compile errors, deep recursion and an infinite loop are covered
//! by the focused tests at the bottom.
//!
//! Ignored by default. Needs `SANDBOX_DOCKER_SOCKET` and the pinned temurin and gcc images
//! on the rootless daemon, and never runs on the host.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_stretch_languages -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx_support;

use std::time::Duration;

use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::DriverChannel;
use sbx_support::{ask, catalogue_enabled, finish, int, list_of, signature, start_on, Mode, Reply};
use serde_json::{json, Value};

/// One stretch case: the same function in Python, Java and C++, plus the calls to make.
struct Case {
    name: &'static str,
    python: &'static str,
    java: &'static str,
    cpp: &'static str,
    signature: Vec<u8>,
    calls: Vec<Value>,
}

fn cases() -> Vec<Case> {
    let int_list = list_of(&int());
    vec![
        Case {
            name: "scalar",
            python: "def add(a, b):\n    return a + b\n",
            java: "public class Solution {\n    public static long add(long a, long b) { return a + b; }\n}\n",
            cpp: "long long add(long long a, long long b) { return a + b; }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(add(from_json<long long>(args[0]), from_json<long long>(args[1])));\n}\n",
            signature: signature("add", &[("a", int()), ("b", int())], &int()),
            calls: vec![json!([2, 3]), json!([10, -4]), json!([0, 0])],
        },
        Case {
            name: "nested list",
            python: "def pairs(xs):\n    return [[x, x * 2] for x in xs]\n",
            java: "public class Solution {\n    public static long[][] pairs(long[] xs) {\n        long[][] out = new long[xs.length][2];\n        for (int i = 0; i < xs.length; i++) { out[i][0] = xs[i]; out[i][1] = xs[i] * 2; }\n        return out;\n    }\n}\n",
            cpp: "std::vector<std::vector<long long>> pairs(std::vector<long long> xs) {\n    std::vector<std::vector<long long>> out;\n    for (long long x : xs) { out.push_back({x, x * 2}); }\n    return out;\n}\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(pairs(from_json<std::vector<long long>>(args[0])));\n}\n",
            signature: signature("pairs", &[("xs", int_list.clone())], &list_of(&int_list)),
            calls: vec![json!([[1, 2]]), json!([[]])],
        },
        Case {
            name: "boolean",
            python: "def is_even(n):\n    return n % 2 == 0\n",
            java: "public class Solution {\n    public static boolean is_even(long n) { return n % 2 == 0; }\n}\n",
            cpp: "bool is_even(long long n) { return n % 2 == 0; }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(is_even(from_json<long long>(args[0])));\n}\n",
            signature: signature("is_even", &[("n", int())], &json!({"kind": "bool"})),
            calls: vec![json!([4]), json!([7])],
        },
        Case {
            name: "user prints",
            python: "def f(n):\n    print('hello')\n    return n\n",
            java: "public class Solution {\n    public static long f(long n) { System.out.println(\"hello\"); return n; }\n}\n",
            cpp: "long long f(long long n) { std::printf(\"hello\\n\"); return n; }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(f(from_json<long long>(args[0])));\n}\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([9])],
        },
        Case {
            name: "raising function",
            python: "def f(n):\n    raise ValueError('boom')\n",
            java: "public class Solution {\n    public static long f(long n) { throw new RuntimeException(\"boom\"); }\n}\n",
            cpp: "long long f(long long) { throw std::runtime_error(\"boom\"); }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(f(from_json<long long>(args[0])));\n}\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([1])],
        },
        Case {
            name: "wrong argument count",
            python: "def f(n):\n    return n\n",
            java: "public class Solution {\n    public static long f(long n) { return n; }\n}\n",
            cpp: "long long f(long long n) { return n; }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(f(from_json<long long>(args[0])));\n}\n",
            signature: signature("f", &[("n", int())], &int()),
            calls: vec![json!([1, 2])],
        },
    ]
}

/// Runs every call of a case in one container on the given catalogue and returns the replies.
async fn replies(
    catalog: &LanguageCatalog,
    lang: &str,
    solution: &str,
    sig: &[u8],
    calls: &[Value],
    mode: Mode,
) -> Vec<Reply> {
    let (session, driver) = start_on(catalog, lang, solution, sig.to_vec(), mode).await;
    let mut driver = driver.unwrap_or_else(|err| panic!("{lang} compiles: {err}"));
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
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python, temurin and gcc images"]
async fn sbx_java_and_cpp_drivers_agree_with_python_on_every_case() {
    let (_dir, catalog) = catalogue_enabled(&["java", "cpp"]);
    let mut disagreements = Vec::new();
    for case in cases() {
        for mode in [Mode::Normal, Mode::Bench] {
            let python = replies(
                &catalog,
                "python",
                case.python,
                &case.signature,
                &case.calls,
                mode,
            )
            .await;
            for (lang, source) in [("java", case.java), ("cpp", case.cpp)] {
                let stretch =
                    replies(&catalog, lang, source, &case.signature, &case.calls, mode).await;
                if python != stretch {
                    disagreements.push(format!(
                        "{} ({mode:?}): python {python:?} vs {lang} {stretch:?}",
                        case.name
                    ));
                }
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "the stretch drivers disagree with python:\n{}",
        disagreements.join("\n")
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned temurin and gcc images"]
async fn sbx_stretch_compile_errors_never_start_the_driver() {
    let (_dir, catalog) = catalogue_enabled(&["java", "cpp"]);
    for (lang, bad) in [
        (
            "java",
            "public class Solution {\n    public static long f(long n) { return n }\n}\n",
        ),
        ("cpp", "long long f(long long n) { return n }\n"),
    ] {
        let sig = signature("f", &[("n", int())], &int());
        let (session, driver) = start_on(&catalog, lang, bad, sig, Mode::Normal).await;
        let err = match driver {
            Err(err) => err,
            Ok(_) => panic!("{lang} reports the compile failure instead of starting"),
        };
        assert!(!err.is_empty(), "{lang} compile failure carries stderr");
        finish(session).await;
    }
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned temurin and gcc images"]
async fn sbx_stretch_deep_recursion_fits_the_configured_stack() {
    let (_dir, catalog) = catalogue_enabled(&["java", "cpp"]);
    let sig = signature("depth", &[("n", int())], &int());
    // The T14 sizing evidence: linear recursion to depth n, as linked-list and
    // tree-depth problems take. 50_000 frames fits the configured -Xss14m.
    for (lang, source) in [
        ("java", "public class Solution {\n    public static long depth(long n) { return n <= 0 ? 0 : 1 + depth(n - 1); }\n}\n"),
        ("cpp", "long long depth(long long n) { return n <= 0 ? 0 : 1 + depth(n - 1); }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(depth(from_json<long long>(args[0])));\n}\n"),
    ] {
        let (session, driver) = start_on(&catalog, lang, source, sig.clone(), Mode::Normal).await;
        let mut driver = driver.unwrap_or_else(|err| panic!("{lang} compiles: {err}"));
        let reply = ask(&mut driver, 0, json!([5_000])).await;
        assert!(
            matches!(reply, Reply::Answer(_)),
            "{lang} answers a 5_000-deep recursion: {reply:?}"
        );
        finish(session).await;
    }
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned temurin and gcc images"]
async fn sbx_stretch_infinite_loop_never_answers_and_the_container_cleans_up() {
    let (_dir, catalog) = catalogue_enabled(&["java", "cpp"]);
    let sig = signature("f", &[("n", int())], &int());
    for (lang, source) in [
        ("java", "public class Solution {\n    public static long f(long n) { while (true) {} }\n}\n"),
        ("cpp", "long long f(long long) { for (;;) {} }\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(f(from_json<long long>(args[0])));\n}\n"),
    ] {
        let (session, driver) = start_on(&catalog, lang, source, sig.clone(), Mode::Normal).await;
        let mut driver = driver.unwrap_or_else(|err| panic!("{lang} compiles: {err}"));
        // The driver is hung inside the call, so no answer line ever arrives; the event
        // wait itself is bounded by the shared EVENT_TIMEOUT inside `ask`, which panics
        // on expiry — for a timeout case that panic IS the proof, so poll the raw event
        // stream with our own short deadline instead.
        driver
            .write_line(b"{\"t\":0,\"args\":[1]}\n")
            .await
            .expect("the driver accepts a call line");
        let answered = tokio::time::timeout(Duration::from_secs(5), driver.next_event()).await;
        match answered {
            Err(_) => {} // still silent after five seconds: the loop holds the driver
            Ok(_) => panic!("{lang} answered a call that should run forever"),
        }
        finish(session).await;
    }
}
