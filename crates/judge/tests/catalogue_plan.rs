//! Import planner tests (problem-catalogue C11, CAT-I7 to CAT-I9). Table-driven, pure.

use counterspell_judge::catalogue::plan::{plan, Incoming, Plan, Refusal, StoredVersion};

fn hash(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn stored(version: i32, byte: u8) -> StoredVersion {
    StoredVersion {
        version,
        hash: hash(byte),
        files: vec![("public/a.txt".to_string(), "aa".to_string())],
    }
}

fn incoming(pack_version: i32, byte: u8, files: &[(&str, &str)]) -> Incoming {
    Incoming {
        pack_version,
        hash: hash(byte),
        files: files
            .iter()
            .map(|(p, h)| ((*p).to_string(), (*h).to_string()))
            .collect(),
    }
}

#[test]
fn no_stored_version_creates() {
    let result = plan(None, &incoming(1, 1, &[]));
    assert_eq!(result, Plan::Create);
}

#[test]
fn same_hash_is_unchanged() {
    let latest = stored(1, 1);
    assert_eq!(plan(Some(&latest), &incoming(1, 1, &[])), Plan::Unchanged);
}

#[test]
fn bigger_version_with_new_hash_creates() {
    let latest = stored(1, 1);
    assert_eq!(plan(Some(&latest), &incoming(2, 2, &[])), Plan::Create);
}

#[test]
fn same_version_with_new_hash_refuses_and_lists_changed_files() {
    let latest = stored(1, 1);
    let result = plan(Some(&latest), &incoming(1, 2, &[("public/a.txt", "bb")]));
    assert_eq!(
        result,
        Plan::Refuse(Refusal::VersionNotBumped {
            changed: vec!["public/a.txt".to_string()]
        })
    );
}

#[test]
fn lower_version_with_new_hash_refuses_as_regressed() {
    let latest = stored(2, 1);
    assert_eq!(
        plan(Some(&latest), &incoming(1, 2, &[])),
        Plan::Refuse(Refusal::VersionRegressed)
    );
}
