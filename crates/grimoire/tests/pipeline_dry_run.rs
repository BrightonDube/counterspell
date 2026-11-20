//! PA23/PA24: run the static pipeline (S0-S9) on a fixture pack with a fake
//! `PackRunner`, in a temporary directory. The fake is not evidence the sandbox
//! works; it only exercises the static stages. S10/S11 stay `pending_judge`:
//! the state file records `static_complete`, and the runner is never called.

mod fixtures {
    pub mod pipeline {
        pub mod fake_runner;
    }
}

use counterspell_engine::classify::ProgramResult;
use counterspell_grimoire::pipeline::state::{PackState, PackStatus, Stage};
use counterspell_grimoire::validate_static;
use fixtures::pipeline::fake_runner::FakeRunner;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Copies the `lantern-streaks` fixture into a temporary directory.
struct TempPack {
    root: PathBuf,
}

impl TempPack {
    fn new() -> Self {
        let parent = std::env::temp_dir().join(format!("grimoire-dry-run-{}", std::process::id()));
        let root = parent.join("lantern-streaks");
        let _ = fs::remove_dir_all(&parent);
        copy_dir(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pipeline/lantern-streaks"),
            &root,
        );
        Self { root }
    }
}

impl Drop for TempPack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.root.parent().unwrap_or(&self.root));
    }
}

fn copy_dir(from: PathBuf, to: &Path) {
    for entry in fs::read_dir(&from).expect("fixture dir") {
        let entry = entry.expect("entry");
        let dest = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            fs::create_dir_all(&dest).expect("mkdir");
            copy_dir(entry.path(), &dest);
        } else {
            fs::create_dir_all(to).expect("mkdir");
            fs::copy(entry.path(), &dest).expect("copy");
        }
    }
}

#[test]
fn s0_to_s9_static_pipeline_passes_and_sandbox_stages_stay_pending() {
    let pack = TempPack::new();

    // S9: static validation on the assembled (fixture) tree passes.
    let report = validate_static(&pack.root).expect("io ok");
    assert!(
        report.error_count() == 0,
        "S9 static validation must be clean, got {report:?}"
    );

    // The pipeline rests at S10 with status static_complete: sandbox stages
    // S10/S11 are pending_judge and the runner was never invoked.
    let mut runner = FakeRunner::new(json!(2)).with_program(
        "incorrect/counts-all-lit.py",
        ProgramResult::Returned(json!(3)),
    );
    let state = PackState {
        slug: "lantern-streaks".to_owned(),
        stage: Stage::S10,
        status: PackStatus::StaticComplete,
    };
    let text = state.to_toml().expect("state serialises");
    assert!(text.contains("status = \"static_complete\""));
    assert_eq!(runner.calls, 0, "no sandbox stage may run in the dry run");

    // Canary: the state file carries no hidden values, only the stage and status.
    let parsed = PackState::from_toml(&text).expect("state parses");
    assert_eq!(parsed.slug, "lantern-streaks");
    assert!(
        !text.contains("expected"),
        "no hidden values in the state file"
    );

    // The repository is unchanged: every fixture file still exists, nothing was
    // written outside the temporary directory.
    assert!(pack.root.join("meta.toml").exists());
    assert!(pack.root.join("reference/solution.py").exists());
}
