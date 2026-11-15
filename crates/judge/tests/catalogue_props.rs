//! Property tests for the catalogue's pure rules (problem-catalogue C06 and later C11a, C12).
//!
//! Each property names the requirement it protects. The fixed case count keeps CI quick and
//! repeatable; proptest writes any failing seed under `proptest-regressions/`.

use counterspell_judge::catalogue::manifest::{build_manifest, Tree};
use counterspell_judge::catalogue::plan::{plan, Incoming, Plan, StoredVersion};
use counterspell_judge::catalogue::retired::canonical_hash;
use proptest::prelude::*;
use serde_json::json;

/// Generates 1 to 5 files with unique names. Each file is public or private and has random bytes.
fn files() -> impl Strategy<Value = Vec<(bool, String, Vec<u8>)>> {
    prop::collection::vec(
        (
            any::<bool>(),
            "[a-z]{1,6}",
            prop::collection::vec(any::<u8>(), 0..24),
        ),
        1..=5,
    )
    .prop_map(|items| {
        items
            .into_iter()
            .enumerate()
            .map(|(i, (private, name, bytes))| (private, format!("{name}{i}.txt"), bytes))
            .collect()
    })
}

fn as_tree(private: bool) -> Tree {
    if private {
        Tree::Private
    } else {
        Tree::Public
    }
}

fn hash_of(items: &[(bool, String, Vec<u8>)]) -> [u8; 32] {
    let borrowed: Vec<(Tree, &str, &[u8])> = items
        .iter()
        .map(|(private, name, bytes)| (as_tree(*private), name.as_str(), bytes.as_slice()))
        .collect();
    *build_manifest(&borrowed)
        .expect("generated names are unique")
        .content_sha256()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// CAT-I6: the hash does not depend on the order the files are given in.
    #[test]
    fn manifest_hash_ignores_input_order(items in files()) {
        let mut reversed = items.clone();
        reversed.reverse();
        prop_assert_eq!(hash_of(&items), hash_of(&reversed));
    }

    /// CAT-I6: a one-byte change to any file changes the hash.
    #[test]
    fn manifest_hash_changes_when_a_byte_changes(items in files()) {
        let mut changed = items.clone();
        changed[0].2.push(0x41);
        prop_assert_ne!(hash_of(&items), hash_of(&changed));
    }

    /// CAT-PT6: renaming a file changes the hash.
    #[test]
    fn manifest_hash_changes_when_a_file_is_renamed(items in files()) {
        let mut renamed = items.clone();
        renamed[0].1 = format!("renamed-{}", renamed[0].1);
        prop_assert_ne!(hash_of(&items), hash_of(&renamed));
    }
}

/// Generates an incoming pack with a small version, a small hash space and up to two files.
/// A small hash space makes repeats (the same content again) likely, which the properties need.
fn incoming() -> impl Strategy<Value = Incoming> {
    (
        1..=4i32,
        0u8..3,
        prop::collection::vec(("[a-z]{1,4}", 0u8..3), 0..3),
    )
        .prop_map(|(pack_version, byte, files)| Incoming {
            pack_version,
            hash: [byte; 32],
            files: files
                .into_iter()
                .map(|(name, digest)| (format!("public/{name}"), format!("{digest:02x}")))
                .collect(),
        })
}

/// Applies `incoming` to the model store the way the importer will: a `Create` appends.
fn apply(store: &mut Vec<StoredVersion>, incoming: &Incoming) -> Plan {
    let outcome = plan(store.last(), incoming);
    if outcome == Plan::Create {
        store.push(StoredVersion {
            version: incoming.pack_version,
            hash: incoming.hash,
            files: incoming.files.clone(),
        });
    }
    outcome
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// CAT-Q1 (P1): planning the same pack twice creates once, then leaves the store unchanged.
    #[test]
    fn importing_the_same_pack_twice_is_idempotent(pack in incoming()) {
        let mut store = Vec::new();
        prop_assert_eq!(apply(&mut store, &pack), Plan::Create);
        prop_assert_eq!(apply(&mut store, &pack), Plan::Unchanged);
        prop_assert_eq!(store.len(), 1);
    }

    /// CAT-I10 (P2): every accepted import adds exactly one version, and earlier versions never change.
    #[test]
    fn accepted_imports_only_append(packs in prop::collection::vec(incoming(), 1..8)) {
        let mut store: Vec<StoredVersion> = Vec::new();
        let mut creates = 0usize;
        for pack in &packs {
            let before = store.clone();
            let outcome = apply(&mut store, pack);
            if outcome == Plan::Create {
                creates += 1;
            }
            prop_assert_eq!(store.len(), creates);
            prop_assert_eq!(&store[..before.len()], &before[..]);
        }
    }

    /// CAT-PT13 / C11a: the canonical hash does not depend on the order keys were written in.
    #[test]
    fn canonical_hash_ignores_object_key_order(a in 0i64..100, b in 0i64..100) {
        let forward = json!({"alpha": a, "beta": b});
        let backward = json!({"beta": b, "alpha": a});
        prop_assert_eq!(canonical_hash(&forward, &json!(1)), canonical_hash(&backward, &json!(1)));
    }
}
