//! Suggested challenges: a deterministic ranking of published, unsolved problems.
//!
//! This is a pure function. The database query supplies the candidates and the learner's weak
//! concepts, and this module decides the order. It does not depend on Axum or PostgreSQL.

use super::types::{ConceptRef, Difficulty, SuggestedChallenge, SuggestionReason};
use std::collections::HashMap;

/// A catalogue problem that the query returned for this learner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The problem's URL-safe slug.
    pub slug: String,
    /// The problem's title.
    pub title: String,
    /// The problem's difficulty.
    pub difficulty: Difficulty,
    /// The problem's concepts.
    pub concepts: Vec<ConceptRef>,
    /// Whether the problem is published in the catalogue.
    pub published: bool,
    /// Whether the learner has already solved the problem.
    pub solved: bool,
}

/// One of the learner's weak concepts, with the number of failed runs that carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeakSignal {
    /// The concept's slug.
    pub slug: String,
    /// How many of the learner's failed runs carry this concept.
    pub failure_count: u32,
}

/// Ranks the candidates and returns at most `limit` suggestions.
///
/// Unpublished and solved problems are dropped first. The rest are ordered by: more shared weak
/// concepts, then a higher failure count on a shared concept, then easier difficulty, then slug.
/// A problem with no shared weak concept gets `ContinueTrack`. Other problems get `PracticeWeakConcept`.
#[must_use]
pub fn rank_suggestions(
    candidates: &[Candidate],
    weak: &[WeakSignal],
    limit: usize,
) -> Vec<SuggestedChallenge> {
    let failures: HashMap<&str, u32> = weak
        .iter()
        .map(|signal| (signal.slug.as_str(), signal.failure_count))
        .collect();
    let mut scored: Vec<Scored<'_>> = candidates
        .iter()
        .filter(|candidate| candidate.published && !candidate.solved)
        .map(|candidate| {
            let shared: Vec<u32> = candidate
                .concepts
                .iter()
                .filter_map(|concept| failures.get(concept.slug.as_str()).copied())
                .collect();
            Scored {
                candidate,
                overlap: shared.len(),
                worst_failures: shared.iter().copied().max().unwrap_or(0),
            }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.overlap
            .cmp(&a.overlap)
            .then(b.worst_failures.cmp(&a.worst_failures))
            .then(a.candidate.difficulty.cmp(&b.candidate.difficulty))
            .then_with(|| a.candidate.slug.cmp(&b.candidate.slug))
    });
    scored
        .into_iter()
        .take(limit)
        .map(Scored::into_suggestion)
        .collect()
}

/// A candidate with its weak-concept overlap, used only while sorting.
struct Scored<'a> {
    candidate: &'a Candidate,
    overlap: usize,
    worst_failures: u32,
}

impl Scored<'_> {
    fn into_suggestion(self) -> SuggestedChallenge {
        let reason = if self.overlap > 0 {
            SuggestionReason::PracticeWeakConcept
        } else {
            SuggestionReason::ContinueTrack
        };
        SuggestedChallenge {
            slug: self.candidate.slug.clone(),
            title: self.candidate.title.clone(),
            difficulty: self.candidate.difficulty,
            concepts: self.candidate.concepts.clone(),
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::types::{ConceptRef, Difficulty, SuggestionReason};

    fn concept(slug: &str, name: &str) -> ConceptRef {
        ConceptRef {
            slug: slug.to_owned(),
            name: name.to_owned(),
        }
    }

    fn candidate(slug: &str, difficulty: Difficulty, concepts: Vec<ConceptRef>) -> Candidate {
        Candidate {
            slug: slug.to_owned(),
            title: format!("Title of {slug}"),
            difficulty,
            concepts,
            published: true,
            solved: false,
        }
    }

    fn weak(slug: &str, failure_count: u32) -> WeakSignal {
        WeakSignal {
            slug: slug.to_owned(),
            failure_count,
        }
    }

    fn slugs(suggestions: &[SuggestedChallenge]) -> Vec<&str> {
        suggestions.iter().map(|s| s.slug.as_str()).collect()
    }

    #[test]
    fn a_problem_sharing_a_weak_concept_comes_before_an_unrelated_easier_one() {
        let candidates = [
            candidate(
                "easy-loops",
                Difficulty::Easy,
                vec![concept("loops", "Loops")],
            ),
            candidate(
                "window-sums",
                Difficulty::Hard,
                vec![concept("sliding-window", "Sliding window")],
            ),
        ];
        let weak_list = [weak("sliding-window", 2)];

        let suggestions = rank_suggestions(&candidates, &weak_list, 3);

        assert_eq!(slugs(&suggestions), ["window-sums", "easy-loops"]);
        assert_eq!(suggestions[0].reason, SuggestionReason::PracticeWeakConcept);
        assert_eq!(suggestions[1].reason, SuggestionReason::ContinueTrack);
    }

    #[test]
    fn a_higher_failure_count_wins_when_overlap_is_equal() {
        let candidates = [
            candidate("low-fail", Difficulty::Easy, vec![concept("a", "A")]),
            candidate("high-fail", Difficulty::Hard, vec![concept("b", "B")]),
        ];
        let weak_list = [weak("a", 1), weak("b", 5)];

        let suggestions = rank_suggestions(&candidates, &weak_list, 3);

        assert_eq!(slugs(&suggestions), ["high-fail", "low-fail"]);
    }

    #[test]
    fn without_weak_concepts_easier_problems_come_first_and_slugs_break_ties() {
        let candidates = [
            candidate("zeta", Difficulty::Easy, vec![]),
            candidate("hard-one", Difficulty::Hard, vec![]),
            candidate("alpha", Difficulty::Easy, vec![]),
        ];

        let suggestions = rank_suggestions(&candidates, &[], 3);

        assert_eq!(slugs(&suggestions), ["alpha", "zeta", "hard-one"]);
    }

    #[test]
    fn solved_and_unpublished_problems_are_never_suggested() {
        let mut solved = candidate("solved-one", Difficulty::Easy, vec![]);
        solved.solved = true;
        let mut hidden = candidate("hidden-one", Difficulty::Easy, vec![]);
        hidden.published = false;
        let candidates = [
            solved,
            hidden,
            candidate("open-one", Difficulty::Medium, vec![]),
        ];

        let suggestions = rank_suggestions(&candidates, &[], 3);

        assert_eq!(slugs(&suggestions), ["open-one"]);
    }

    #[test]
    fn at_most_the_limit_is_returned_and_fewer_candidates_give_fewer() {
        let candidates = [
            candidate("p1", Difficulty::Easy, vec![]),
            candidate("p2", Difficulty::Easy, vec![]),
            candidate("p3", Difficulty::Easy, vec![]),
            candidate("p4", Difficulty::Easy, vec![]),
        ];

        assert_eq!(rank_suggestions(&candidates, &[], 3).len(), 3);
        assert_eq!(rank_suggestions(&candidates[..2], &[], 3).len(), 2);
        assert_eq!(rank_suggestions(&[], &[], 3), Vec::new());
    }

    #[test]
    fn repeated_calls_with_the_same_input_give_the_same_order() {
        let candidates = [
            candidate("c", Difficulty::Medium, vec![concept("x", "X")]),
            candidate("a", Difficulty::Medium, vec![concept("x", "X")]),
            candidate("b", Difficulty::Medium, vec![concept("x", "X")]),
        ];
        let weak_list = [weak("x", 3)];

        let first = rank_suggestions(&candidates, &weak_list, 3);
        let second = rank_suggestions(&candidates, &weak_list, 3);

        assert_eq!(first, second);
        assert_eq!(slugs(&first), ["a", "b", "c"]);
    }
}
