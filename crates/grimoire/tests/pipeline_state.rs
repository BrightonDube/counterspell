//! PA01 (PIP-6, PIP-45, P10): the pack state file and the status transitions of design §7.
//!
//! The state file is parsed from text. The test never touches the disk.

use counterspell_grimoire::pipeline::state::{PackState, PackStatus, Stage};

const VALID: &str = "slug = \"count-pairs\"\nstage = \"S9\"\nstatus = \"static_complete\"\n";

#[test]
fn parses_slug_stage_and_status_from_a_state_file() {
    let state = PackState::from_toml(VALID).expect("a valid state file parses");

    assert_eq!(state.slug, "count-pairs");
    assert_eq!(state.stage, Stage::S9);
    assert_eq!(state.status, PackStatus::StaticComplete);
}

#[test]
fn a_hidden_key_makes_the_state_file_fail_to_parse() {
    let text = format!("{VALID}hidden_expected = \"[0, 1]\"\n");

    assert!(PackState::from_toml(&text).is_err());
}

#[test]
fn published_is_not_a_pipeline_status() {
    let text = "slug = \"count-pairs\"\nstage = \"S13\"\nstatus = \"published\"\n";

    assert!(PackState::from_toml(text).is_err());
}

#[test]
fn stage_beyond_s13_is_refused() {
    let text = "slug = \"count-pairs\"\nstage = \"S14\"\nstatus = \"in_progress\"\n";

    assert!(PackState::from_toml(text).is_err());
}

#[test]
fn writing_then_parsing_keeps_the_same_state() {
    let state = PackState::from_toml(VALID).expect("a valid state file parses");

    let written = state.to_toml().expect("state serialises");
    let again = PackState::from_toml(&written).expect("written state parses");

    assert_eq!(again, state);
    assert!(written.contains("status = \"static_complete\""));
}

#[test]
fn every_allowed_transition_of_design_section_7_is_accepted() {
    let allowed = [
        (PackStatus::InProgress, PackStatus::StaticComplete),
        (PackStatus::StaticComplete, PackStatus::SandboxChecked),
        (PackStatus::SandboxChecked, PackStatus::InReview),
        (PackStatus::InReview, PackStatus::Rework),
        (PackStatus::InReview, PackStatus::Escalated),
        (PackStatus::InReview, PackStatus::Stale),
        (PackStatus::InReview, PackStatus::Passed),
        (PackStatus::Rework, PackStatus::InReview),
        (PackStatus::Stale, PackStatus::InReview),
        (PackStatus::Passed, PackStatus::AwaitingApproval),
        (PackStatus::AwaitingApproval, PackStatus::Approved),
        (PackStatus::AwaitingApproval, PackStatus::Rework),
    ];
    for (from, to) in allowed {
        let mut state = state_at(from);

        state
            .move_to(to)
            .unwrap_or_else(|error| panic!("{from:?} -> {to:?} should be allowed: {error}"));

        assert_eq!(state.status, to);
    }
}

#[test]
fn every_other_transition_is_refused_and_leaves_the_status_unchanged() {
    let everything = [
        PackStatus::InProgress,
        PackStatus::StaticComplete,
        PackStatus::SandboxChecked,
        PackStatus::InReview,
        PackStatus::Rework,
        PackStatus::Stale,
        PackStatus::Passed,
        PackStatus::Escalated,
        PackStatus::AwaitingApproval,
        PackStatus::Approved,
    ];
    let allowed_count = 12;
    let mut refused = 0;
    for from in everything {
        for to in everything {
            let mut state = state_at(from);
            if state.move_to(to).is_err() {
                refused += 1;
                assert_eq!(
                    state.status, from,
                    "a refused move must not change the status"
                );
            }
        }
    }

    assert_eq!(refused, everything.len() * everything.len() - allowed_count);
}

#[test]
fn approved_cannot_move_back_to_review() {
    let mut state = state_at(PackStatus::Approved);

    assert!(state.move_to(PackStatus::InReview).is_err());
    assert_eq!(state.status, PackStatus::Approved);
}

fn state_at(status: PackStatus) -> PackState {
    PackState {
        slug: "count-pairs".to_owned(),
        stage: Stage::S12,
        status,
    }
}
