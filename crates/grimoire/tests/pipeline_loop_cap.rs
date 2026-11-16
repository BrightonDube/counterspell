//! PA03 (PIP-25, PIP-26, P4): the review loop allows three rounds, then escalates.

use counterspell_grimoire::pipeline::loop_cap::{LoopError, ReviewLoop};
use counterspell_grimoire::pipeline::state::PackStatus;

const HASH_A: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const HASH_B: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const HASH_C: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const HASH_D: &str = "4444444444444444444444444444444444444444444444444444444444444444";

#[test]
fn rounds_one_and_two_send_a_failed_pack_back_to_rework() {
    let mut loop_state = ReviewLoop::new();

    assert_eq!(loop_state.start_round(HASH_A), Ok(1));
    assert_eq!(loop_state.finish_round(false), Ok(PackStatus::Rework));

    assert_eq!(loop_state.start_round(HASH_B), Ok(2));
    assert_eq!(loop_state.finish_round(false), Ok(PackStatus::Rework));
}

#[test]
fn round_three_that_fails_escalates_the_pack() {
    let mut loop_state = ReviewLoop::new();
    loop_state.start_round(HASH_A).expect("round 1");
    loop_state.start_round(HASH_B).expect("round 2");

    assert_eq!(loop_state.start_round(HASH_C), Ok(3));
    assert_eq!(loop_state.finish_round(false), Ok(PackStatus::Escalated));
}

#[test]
fn a_fourth_round_on_a_new_hash_is_refused() {
    let mut loop_state = ReviewLoop::new();
    loop_state.start_round(HASH_A).expect("round 1");
    loop_state.start_round(HASH_B).expect("round 2");
    loop_state.start_round(HASH_C).expect("round 3");

    assert_eq!(
        loop_state.start_round(HASH_D),
        Err(LoopError::RoundLimitReached)
    );
    assert_eq!(loop_state.round(), 3);
}

#[test]
fn rerunning_the_same_hash_does_not_increase_the_round() {
    let mut loop_state = ReviewLoop::new();

    assert_eq!(loop_state.start_round(HASH_A), Ok(1));
    assert_eq!(loop_state.start_round(HASH_A), Ok(1));
    assert_eq!(loop_state.start_round(HASH_A), Ok(1));
    assert_eq!(loop_state.round(), 1);
}

#[test]
fn a_passing_round_gives_passed() {
    let mut loop_state = ReviewLoop::new();
    loop_state.start_round(HASH_A).expect("round 1");

    assert_eq!(loop_state.finish_round(true), Ok(PackStatus::Passed));
}

#[test]
fn a_hash_that_changed_since_the_round_started_is_stale() {
    let mut loop_state = ReviewLoop::new();
    loop_state.start_round(HASH_A).expect("round 1");

    assert!(loop_state.is_stale(HASH_B));
    assert!(!loop_state.is_stale(HASH_A));
}

#[test]
fn finishing_before_any_round_is_an_error() {
    let loop_state = ReviewLoop::new();

    assert_eq!(
        loop_state.finish_round(true),
        Err(LoopError::NoRoundStarted)
    );
}
