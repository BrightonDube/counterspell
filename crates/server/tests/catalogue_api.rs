//! Tests for the public catalogue list that need no database (problem-catalogue C30, CAT-A1 to
//! CAT-A3 and CAT-A8).
//!
//! The filter checks return before any query runs, so they use a pool that never connects. The key
//! allow-list compares the JSON keys of the explicit response structs with the keys the spec allows.

mod support;

use std::collections::BTreeSet;

use counterspell_server::catalogue::dto::{
    ConceptRef, ProblemListResponse, ProblemSummary, SolveStatus, TrackRef,
};
use serde_json::{json, Value};
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
