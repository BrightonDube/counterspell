//! Property tests for the catalogue's pure rules (problem-catalogue C06 and later C11a, C12).
//!
//! Each property names the requirement it protects. The fixed case count keeps CI quick and
//! repeatable; proptest writes any failing seed under `proptest-regressions/`.

use counterspell_judge::catalogue::manifest::{build_manifest, Tree};
use proptest::prelude::*;

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
