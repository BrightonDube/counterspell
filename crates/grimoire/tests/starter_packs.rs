//! Every starter pack in `packs/` passes the static checks with zero errors (GRM-52).
//!
//! The starter list is written out here by hand, so a pack that goes missing or
//! is renamed fails this test, not just a pack that is invalid.

use counterspell_grimoire::validate_static_pair;
use std::path::{Path, PathBuf};

/// The three starter slugs from the grimoire-packs design, in order.
const STARTER_SLUGS: [&str; 3] = ["count-pairs", "diagonal-total", "longest-run-length"];

/// The `packs/` directory at the repository root.
fn packs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs")
}

/// The private pack-tree checkout: `COUNTERSPELL_PACKS_PRIVATE_ROOT`, or a
/// `counterspell-packs-private` checkout beside this repository. `reference/`
/// and `incorrect/` live there since the split, so full validation needs it.
fn private_packs_root() -> Option<PathBuf> {
    std::env::var("COUNTERSPELL_PACKS_PRIVATE_ROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let sibling = packs_root().join("../../counterspell-packs-private");
            sibling.is_dir().then_some(sibling)
        })
        .filter(|root| root.join("packs").is_dir())
}

#[test]
fn all_designated_starters_remain_in_the_catalogue() {
    let root = packs_root();
    for slug in STARTER_SLUGS {
        assert!(
            root.join(slug).is_dir(),
            "missing original starter pack {slug}"
        );
    }
}

#[test]
fn every_starter_pack_has_zero_static_errors() {
    let Some(private_root) = private_packs_root() else {
        eprintln!("skipping: no counterspell-packs-private checkout is available");
        return;
    };
    for slug in STARTER_SLUGS {
        let report = validate_static_pair(
            &packs_root().join(slug),
            &private_root.join("packs").join(slug),
        )
        .expect("starter pack trees are readable");

        let lines: Vec<String> = report
            .diagnostics()
            .iter()
            .map(counterspell_grimoire::report::Diagnostic::to_text_line)
            .collect();
        assert!(!report.has_errors(), "{slug} has errors: {lines:?}");
    }
}
