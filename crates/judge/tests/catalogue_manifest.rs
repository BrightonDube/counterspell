//! Canonical manifest and content hash tests (problem-catalogue C06, CAT-I6, CAT-PT6, CAT-PT9).
//!
//! The expected digest in `two_file_manifest_matches_hand_computed_hash` was computed with
//! `sha256sum` from the canonical bytes, not by the code under test:
//!   printf 'public/a.txt\0<sha256 of hello>\npublic/b.txt\0<sha256 of world>\n' | sha256sum
//!   sha256 of hello = 2cf24dba…9824, sha256 of world = 486ea462…c8c3… (see the test)

use counterspell_judge::catalogue::manifest::{build_manifest, ManifestError, Tree};

const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
const WORLD_SHA256: &str = "486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7";
const TWO_FILE_CONTENT_SHA256: &str =
    "f5178ca9e71e0d918035b8f51e64ec24fb6b724a587d804b07b1ea51e930c0bb";

#[test]
fn two_file_manifest_matches_hand_computed_hash() {
    let manifest = build_manifest(&[
        (Tree::Public, "b.txt", b"world"),
        (Tree::Public, "a.txt", b"hello"),
    ])
    .expect("two distinct files must build");

    assert_eq!(manifest.content_sha256_hex(), TWO_FILE_CONTENT_SHA256);
    let paths: Vec<&str> = manifest.entries().iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        ["public/a.txt", "public/b.txt"],
        "entries sort by prefixed path"
    );
    assert_eq!(manifest.entries()[0].sha256_hex, HELLO_SHA256);
    assert_eq!(manifest.entries()[1].sha256_hex, WORLD_SHA256);
}

#[test]
fn approval_file_does_not_change_the_hash() {
    let without = build_manifest(&[(Tree::Public, "a.txt", b"hello")]).expect("build");
    let with = build_manifest(&[
        (Tree::Public, "a.txt", b"hello"),
        (Tree::Public, "approval.toml", b"slug = \"two-sum\"\n"),
    ])
    .expect("build");
    assert_eq!(without.content_sha256(), with.content_sha256());
}

#[test]
fn moving_a_file_to_the_private_tree_changes_the_hash() {
    let public = build_manifest(&[(Tree::Public, "a.txt", b"hello")]).expect("build");
    let private = build_manifest(&[(Tree::Private, "a.txt", b"hello")]).expect("build");
    assert_ne!(public.content_sha256(), private.content_sha256());
}

#[test]
fn public_map_holds_only_public_paths() {
    let manifest = build_manifest(&[
        (Tree::Public, "a.txt", b"hello"),
        (Tree::Private, "secret.toml", b"CANARY-HIDDEN-7f3a"),
    ])
    .expect("build");
    let map = manifest.public_map();
    assert_eq!(
        map.len(),
        1,
        "private entries must not reach the public map"
    );
    assert_eq!(
        map.get("public/a.txt").map(String::as_str),
        Some(HELLO_SHA256)
    );
    assert!(!map.keys().any(|k| k.starts_with("private/")));
}

#[test]
fn duplicate_path_in_one_tree_is_refused() {
    let error = build_manifest(&[
        (Tree::Public, "a.txt", b"hello"),
        (Tree::Public, "a.txt", b"world"),
    ])
    .expect_err("two files with the same path must be refused");
    assert_eq!(
        error,
        ManifestError::DuplicatePath("public/a.txt".to_string())
    );
}
