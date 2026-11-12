//! Every starter pack in `packs/` passes the static checks with zero errors (GRM-52).
//!
//! The starter list is written out here by hand, so a pack that goes missing or
//! is renamed fails this test, not just a pack that is invalid.

use counterspell_grimoire::validate_static;
use std::fs;
use std::path::{Path, PathBuf};

/// The three starter slugs from the grimoire-packs design, in order.
const STARTER_SLUGS: [&str; 3] = ["count-pairs", "diagonal-total", "longest-run-length"];

/// The `packs/` directory at the repository root.
fn packs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs")
}

/// The non-underscore pack directories under `root`, sorted by name.
fn starter_dirs(root: &Path) -> Vec<String> {
    let mut slugs: Vec<String> = fs::read_dir(root)
        .expect("packs directory is readable")
        .map(|entry| {
            entry
                .expect("directory entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| !name.starts_with('_'))
        .filter(|name| root.join(name).is_dir())
        .collect();
    slugs.sort();
    slugs
}

#[test]
fn starter_pack_list_is_exactly_the_three_design_slugs() {
    assert_eq!(starter_dirs(&packs_root()), STARTER_SLUGS);
}

#[test]
fn every_starter_pack_has_zero_static_errors() {
    for slug in STARTER_SLUGS {
        let report =
            validate_static(&packs_root().join(slug)).expect("starter pack directory is readable");

        let lines: Vec<String> = report
            .diagnostics()
            .iter()
            .map(counterspell_grimoire::report::Diagnostic::to_text_line)
            .collect();
        assert!(!report.has_errors(), "{slug} has errors: {lines:?}");
    }
}
