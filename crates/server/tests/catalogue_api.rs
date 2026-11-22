//! Tests for the public catalogue list that need no database (problem-catalogue C30, CAT-A1 to
//! CAT-A3 and CAT-A8).
//!
//! The filter checks return before any query runs, so they use a pool that never connects. The key
//! allow-list compares the JSON keys of the explicit response structs with the keys the spec allows.

mod support;

use std::collections::BTreeSet;

use counterspell_server::catalogue::dto::{
    ConceptRef, Example, LimitsView, ParameterView, ProblemDetail, ProblemListResponse,
    ProblemSummary, SignatureView, SolveStatus, TrackRef,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use support::{get, serve, state_for, Fixture};

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("a JSON object")
        .keys()
        .cloned()
        .collect()
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn sample_summary(status: Option<SolveStatus>) -> ProblemSummary {
    ProblemSummary {
        slug: "count-pairs".to_owned(),
        title: "Count the Pairs".to_owned(),
        track: TrackRef {
            slug: "arrays".to_owned(),
            name: "Arrays".to_owned(),
        },
        difficulty: "easy".to_owned(),
        concepts: vec![ConceptRef {
            slug: "pairs".to_owned(),
            name: "Pairs".to_owned(),
        }],
        status,
    }
}

#[tokio::test]
async fn list_refuses_an_unknown_difficulty_with_invalid_filter() {
    let fixture = Fixture::new();
    let state = state_for(&fixture.dir);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(address, "/api/problems?difficulty=intro", &[]).await;

    assert_eq!(reply.status, 400);
    let body: Value = serde_json::from_str(&reply.body).expect("JSON body");
    assert_eq!(body["code"], "invalid_filter");
}

#[tokio::test]
async fn list_refuses_a_track_that_is_not_a_slug_with_invalid_filter() {
    let fixture = Fixture::new();
    let state = state_for(&fixture.dir);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(address, "/api/problems?track=Bad_Slug", &[]).await;

    assert_eq!(reply.status, 400);
    let body: Value = serde_json::from_str(&reply.body).expect("JSON body");
    assert_eq!(body["code"], "invalid_filter");
}

#[test]
fn list_item_keys_match_the_allow_list_exactly() {
    let response = ProblemListResponse {
        problems: vec![sample_summary(Some(SolveStatus::Solved))],
    };

    let value = serde_json::to_value(&response).expect("serialises");

    assert_eq!(keys(&value), set(&["problems"]));
    assert_eq!(
        keys(&value["problems"][0]),
        set(&["slug", "title", "track", "difficulty", "concepts", "status"])
    );
    assert_eq!(keys(&value["problems"][0]["track"]), set(&["slug", "name"]));
    assert_eq!(
        keys(&value["problems"][0]["concepts"][0]),
        set(&["slug", "name"])
    );
}

fn detail_sample() -> ProblemDetail {
    ProblemDetail {
        slug: "count-pairs".to_owned(),
        title: "Count the Pairs".to_owned(),
        track: TrackRef {
            slug: "arrays".to_owned(),
            name: "Arrays".to_owned(),
        },
        difficulty: "easy".to_owned(),
        concepts: vec![ConceptRef {
            slug: "pairs".to_owned(),
            name: "Pairs".to_owned(),
        }],
        statement_markdown: "# Count the Pairs".to_owned(),
        constraints: vec!["At most 10 items.".to_owned()],
        examples: vec![Example {
            input: json!([[1, 2], 3]),
            expected: json!(1),
        }],
        signature: SignatureView {
            function_name: "count_pairs".to_owned(),
            parameters: vec![ParameterView {
                name: "values".to_owned(),
                kind: "list[int]".to_owned(),
            }],
            return_type: "int".to_owned(),
        },
        starters: BTreeMap::from([("python".to_owned(), "def count_pairs(values):".to_owned())]),
        enabled_languages: vec!["python".to_owned()],
        limits: LimitsView {
            time_ms: 2000,
            mem_mb: 256,
        },
        hint_count: 4,
    }
}

#[test]
fn detail_keys_match_the_allow_list_exactly() {
    let value = serde_json::to_value(detail_sample()).expect("serialises");

    assert_eq!(
        keys(&value),
        set(&[
            "slug",
            "title",
            "track",
            "difficulty",
            "concepts",
            "statement_markdown",
            "constraints",
            "examples",
            "signature",
            "starters",
            "enabled_languages",
            "limits",
            "hint_count",
        ])
    );
    assert_eq!(keys(&value["examples"][0]), set(&["input", "expected"]));
    assert_eq!(
        keys(&value["signature"]),
        set(&["function_name", "parameters", "return_type"])
    );
    assert_eq!(
        keys(&value["signature"]["parameters"][0]),
        set(&["name", "type"])
    );
    assert_eq!(keys(&value["limits"]), set(&["time_ms", "mem_mb"]));
}

#[test]
fn anonymous_items_carry_a_null_status() {
    let value = serde_json::to_value(sample_summary(None)).expect("serialises");

    assert_eq!(value["status"], json!(null));
}

#[test]
fn statuses_serialise_as_the_spec_words() {
    assert_eq!(
        serde_json::to_value(SolveStatus::Solved).expect("serialises"),
        json!("solved")
    );
    assert_eq!(
        serde_json::to_value(SolveStatus::Attempted).expect("serialises"),
        json!("attempted")
    );
    assert_eq!(
        serde_json::to_value(SolveStatus::Unattempted).expect("serialises"),
        json!("unattempted")
    );
}

// --- C32: GET /api/languages -----------------------------------------------

#[test]
fn language_keys_match_the_allow_list_exactly() {
    let value = serde_json::to_value(counterspell_server::catalogue::dto::LanguageListResponse {
        languages: vec![counterspell_server::catalogue::dto::LanguageSummary {
            id: "python".to_owned(),
            display_name: "Python".to_owned(),
            max_source_bytes: 65_536,
        }],
    })
    .expect("serialises");

    assert_eq!(keys(&value), set(&["languages"]));
    assert_eq!(
        keys(&value["languages"][0]),
        set(&["id", "display_name", "max_source_bytes"])
    );
}

#[test]
fn launch_catalogue_lists_the_accepted_languages() {
    let catalogue = counterspell_server::seams::LanguageCatalog::launch(65_536);
    let ids: Vec<&str> = catalogue
        .entries()
        .iter()
        .map(|language| language.name.as_str())
        .collect();

    assert_eq!(ids, ["python", "javascript", "java", "cpp"]);
    // Languages the server does not accept are never in the catalogue (CAT-A7).
    assert!(catalogue.find("rust").is_none());
}

#[tokio::test]
async fn get_languages_returns_the_enabled_languages() {
    let fixture = Fixture::new();
    let address = serve(counterspell_server::build_router(state_for(&fixture.dir))).await;

    let reply = get(address, "/api/languages", &[]).await;
    assert_eq!(reply.status, 200);
    let value: Value = serde_json::from_str(&reply.body).expect("JSON body");

    assert_eq!(keys(&value), set(&["languages"]));
    let languages = value["languages"].as_array().expect("a languages array");
    assert_eq!(languages.len(), 4);
    assert_eq!(languages[0]["id"], json!("python"));
    assert_eq!(languages[0]["display_name"], json!("Python"));
    assert_eq!(languages[1]["id"], json!("javascript"));
    assert_eq!(languages[1]["display_name"], json!("JavaScript"));
    assert_eq!(languages[2]["id"], json!("java"));
    assert_eq!(languages[2]["display_name"], json!("Java"));
    assert_eq!(languages[3]["id"], json!("cpp"));
    assert_eq!(languages[3]["display_name"], json!("C++"));
    for language in languages {
        assert_eq!(
            keys(language),
            set(&["id", "display_name", "max_source_bytes"])
        );
        assert!(language["max_source_bytes"].as_u64().unwrap() > 0);
    }
}
