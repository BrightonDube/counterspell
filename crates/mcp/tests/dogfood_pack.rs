//! The `_demo/mcp-dogfood` pack contract (mcp-power T15).
//!
//! The public tree carries only problem text, meta and hints; the correct
//! reference, the deliberate bug and the known ghost live in the private
//! checkout. The catalogue importer must skip the underscore pack, and the
//! merged pair must validate when a private root is available.

use counterspell_grimoire::{validate_static, validate_static_pair};
use counterspell_mcp::paths::private_pack_dir;
use counterspell_mcp::schemas::ValidatePackArgs;
use counterspell_mcp::tools::validate_pack;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

#[test]
fn the_public_dogfood_tree_carries_no_hidden_files() {
    let pack = repo_root().join("packs/_demo/mcp-dogfood");
    assert!(pack.join("problem.md").is_file());
    assert!(pack.join("meta.toml").is_file());
    assert!(pack.join("hints.toml").is_file());
    assert!(!pack.join("reference").exists());
    assert!(!pack.join("incorrect").exists());
    assert!(!pack.join("known_ghosts.toml").exists());
}

#[test]
fn public_only_validation_fails_only_on_the_missing_private_parts() {
    let pack = repo_root().join("packs/_demo/mcp-dogfood");
    let report = validate_static(&pack).expect("report");
    assert!(report.has_errors());
    let codes: Vec<String> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect();
    assert!(
        codes
            .iter()
            .all(|code| code.starts_with("LAY") || code == "REF-01"),
        "public-only errors must all be missing-private-part diagnostics, got {codes:?}"
    );
}

#[test]
fn the_merged_pair_validates_and_the_tool_reports_it() {
    let root = repo_root();
    let pack = root.join("packs/_demo/mcp-dogfood");
    let Some(private_dir) = private_pack_dir(&pack) else {
        eprintln!("no private checkout reachable; pair leg skipped");
        return;
    };
    let report = validate_static_pair(&pack, &private_dir).expect("pair report");
    assert!(
        !report.has_errors(),
        "merged dogfood pair must validate: {report:?}"
    );
    assert!(private_dir.join("reference/solution.py").is_file());
    assert!(private_dir.join("incorrect/never-negative.py").is_file());
    assert!(private_dir.join("known_ghosts.toml").is_file());
    let output = validate_pack(
        &root,
        &ValidatePackArgs {
            pack_path: "packs/_demo/mcp-dogfood".to_string(),
            deny_warnings: false,
        },
    )
    .expect("tool result");
    assert_eq!(output["passed"], true);
}
