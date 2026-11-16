//! PA02 (PIP-22, PIP-23, PIP-24): the review pass rule at each boundary.
//!
//! Scores are seven integers in the order correctness, clarity, originality,
//! difficulty, hints, ghosts, accessibility. Seven integers cannot give a mean of
//! exactly 3.9, so the mean boundary is tested with sums 27 and 28 (means 3.86 and 4.0).

use counterspell_grimoire::pipeline::review::{
    check_answer, judge, Answer, Category, Finding, Outcome, ReviewError, Rule, Scores, Severity,
};

const HASH: &str = "a3f1c2d4e5b6a7980123456789abcdef0123456789abcdef0123456789abcdef";

/// A finding with no grimoire code.
fn finding(severity: Severity, category: Category, message: &str) -> Finding {
    Finding {
        id: "F1-1".to_owned(),
        category,
        severity,
        pointer: "hints.toml rung 3".to_owned(),
        message: message.to_owned(),
        code: None,
    }
}

fn scores(values: [u8; 7]) -> Scores {
    Scores::new(values).expect("scores in 1 to 5")
}

fn fail_rules(outcome: Outcome) -> Vec<Rule> {
    match outcome {
        Outcome::Fail(rules) => rules,
        Outcome::Pass => panic!("expected a fail, got a pass"),
    }
}

#[test]
fn all_fives_with_no_findings_pass() {
    let verdict = judge(HASH, &scores([5, 5, 5, 5, 5, 5, 5]), &[]).expect("valid input");

    assert_eq!(verdict.outcome, Outcome::Pass);
    assert_eq!(verdict.content_sha256, HASH);
}

#[test]
fn a_category_at_two_fails_with_that_rule() {
    let verdict = judge(HASH, &scores([5, 5, 5, 5, 2, 5, 5]), &[]).expect("valid input");

    assert_eq!(
        fail_rules(verdict.outcome),
        vec![Rule::CategoryBelowThree(Category::HintQuality)]
    );
}

#[test]
fn a_category_at_three_is_the_lowest_allowed_score() {
    let verdict = judge(HASH, &scores([5, 5, 5, 3, 5, 5, 5]), &[]).expect("valid input");

    assert_eq!(verdict.outcome, Outcome::Pass);
}

#[test]
fn a_sum_of_27_is_a_mean_below_four_and_fails() {
    // Mean 27 / 7 = 3.86. Correctness is 5, originality is 4, every category is 3 or more.
    let verdict = judge(HASH, &scores([5, 4, 4, 4, 4, 3, 3]), &[]).expect("valid input");

    assert_eq!(fail_rules(verdict.outcome), vec![Rule::MeanBelowFour]);
}

#[test]
fn a_sum_of_28_is_a_mean_of_exactly_four_and_passes() {
    let verdict = judge(HASH, &scores([5, 4, 4, 4, 4, 4, 3]), &[]).expect("valid input");

    assert_eq!(verdict.outcome, Outcome::Pass);
}

#[test]
fn correctness_of_four_fails_even_with_a_high_mean() {
    let verdict = judge(HASH, &scores([4, 5, 5, 5, 5, 5, 5]), &[]).expect("valid input");

    assert_eq!(fail_rules(verdict.outcome), vec![Rule::CorrectnessNotFive]);
}

#[test]
fn originality_of_three_fails_the_originality_rule() {
    let verdict = judge(HASH, &scores([5, 5, 3, 5, 5, 5, 5]), &[]).expect("valid input");

    assert_eq!(
        fail_rules(verdict.outcome),
        vec![Rule::OriginalityBelowFour]
    );
}

#[test]
fn an_originality_finding_with_an_org_code_fails() {
    let mut org = finding(Severity::Minor, Category::Originality, "text marker found");
    org.code = Some("ORG-01".to_owned());

    let verdict = judge(HASH, &scores([5, 5, 5, 5, 5, 5, 5]), &[org]).expect("valid input");

    assert_eq!(fail_rules(verdict.outcome), vec![Rule::OriginalityFinding]);
}

#[test]
fn a_blocker_finding_fails() {
    let blocker = finding(Severity::Blocker, Category::Clarity, "answer is unclear");

    let verdict = judge(HASH, &scores([5, 5, 5, 5, 5, 5, 5]), &[blocker]).expect("valid input");

    assert_eq!(fail_rules(verdict.outcome), vec![Rule::BlockerFinding]);
}

#[test]
fn major_and_minor_findings_do_not_fail_the_round_on_their_own() {
    let findings = [
        finding(
            Severity::Major,
            Category::Clarity,
            "reword the second sentence",
        ),
        finding(Severity::Minor, Category::HintQuality, "shorten rung 2"),
    ];

    let verdict = judge(HASH, &scores([5, 5, 5, 5, 5, 5, 5]), &findings).expect("valid input");

    assert_eq!(verdict.outcome, Outcome::Pass);
}

#[test]
fn a_score_of_zero_or_six_is_refused() {
    assert_eq!(
        Scores::new([0, 5, 5, 5, 5, 5, 5]),
        Err(ReviewError::ScoreOutOfRange(Category::Correctness))
    );
    assert_eq!(
        Scores::new([5, 5, 5, 5, 5, 5, 6]),
        Err(ReviewError::ScoreOutOfRange(
            Category::AccessibilityOfWording
        ))
    );
}

#[test]
fn a_content_hash_that_is_not_64_lowercase_hex_is_refused() {
    let short = "abc123";

    assert_eq!(
        judge(short, &scores([5, 5, 5, 5, 5, 5, 5]), &[]),
        Err(ReviewError::InvalidHash)
    );
}

#[test]
fn an_empty_or_blank_answer_is_refused() {
    assert_eq!(
        check_answer(&Answer::Fixed("   ".to_owned())),
        Err(ReviewError::EmptyAnswer)
    );
    assert_eq!(
        check_answer(&Answer::Rebutted(String::new())),
        Err(ReviewError::EmptyAnswer)
    );
}

#[test]
fn a_fixed_or_rebutted_answer_with_text_is_accepted() {
    assert!(check_answer(&Answer::Fixed("9c2e4f1".to_owned())).is_ok());
    assert!(check_answer(&Answer::Rebutted("the rung is general".to_owned())).is_ok());
}

#[test]
fn a_fail_names_the_rule_and_never_repeats_the_finding_message() {
    let canary = "CANARY-HIDDEN-pa2";
    let blocker = finding(Severity::Blocker, Category::Correctness, canary);

    let verdict = judge(HASH, &scores([5, 5, 5, 5, 5, 5, 5]), &[blocker]).expect("valid input");
    let rules = fail_rules(verdict.outcome);
    let text: String = rules.iter().map(ToString::to_string).collect();

    assert!(
        !text.contains(canary),
        "the rule text must not carry the finding message"
    );
}
