//! Enumerations for run lifecycle, verdicts, test results and job states.
//!
//! Each enum stores its text form in Postgres and on the wire. The text is
//! written once, next to the variant, so `as_str`, `parse` and the serde form
//! cannot disagree.

/// Declares a fieldless enum with its text form, the full list of values and
/// a `parse` function that accepts only exact matches.
macro_rules! text_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $text:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $( $(#[$vmeta])* #[serde(rename = $text)] $variant, )+
        }

        impl $name {
            /// Every value, in declaration order.
            pub const ALL: &'static [Self] = &[ $( Self::$variant ),+ ];

            /// The text stored in Postgres and sent to clients.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $text, )+
                }
            }

            /// Reads a stored value back. Unknown or differently cased text gives `None`.
            #[must_use]
            pub fn parse(text: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|value| value.as_str() == text)
            }
        }
    };
}

text_enum! {
    /// Where a run is in its life. Four of these are terminal.
    pub enum RunStatus {
        /// Saved, waiting for a judge.
        Queued => "queued",
        /// A judge holds the lease and is loading the run.
        Claimed => "claimed",
        /// The program is being prepared.
        Compiling => "compiling",
        /// Tests are running.
        Running => "running",
        /// The ghost engine is searching for a failing input.
        GhostHunting => "ghost_hunting",
        /// The judge produced a verdict.
        Completed => "completed",
        /// Ended without a verdict for a fixed reason.
        Failed => "failed",
        /// Waited in the queue past its time to live and was never run.
        Expired => "expired",
        /// Infrastructure failed and the retries ran out. Never a verdict on the code.
        InternalError => "internal_error",
    }
}

impl RunStatus {
    /// True when no further change is allowed.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Expired | Self::InternalError
        )
    }

    /// True while a judge holds the run.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Claimed | Self::Compiling | Self::Running | Self::GhostHunting
        )
    }
}

text_enum! {
    /// The judge's decision on a finished program.
    pub enum Verdict {
        /// Every test passed.
        Accepted => "accepted",
        /// An output did not match the expected one.
        WrongAnswer => "wrong_answer",
        /// The program crashed.
        RuntimeError => "runtime_error",
        /// A test ran past its time limit.
        TimeLimit => "time_limit",
        /// The program ran out of memory.
        MemoryLimit => "memory_limit",
        /// The program printed too much.
        OutputLimit => "output_limit",
        /// The program did not compile.
        CompileError => "compile_error",
    }
}

text_enum! {
    /// The result of one test.
    pub enum TestVerdict {
        /// The output matched.
        Pass => "pass",
        /// The output did not match.
        WrongAnswer => "wrong_answer",
        /// The program crashed on this test.
        RuntimeError => "runtime_error",
        /// This test ran past its time limit.
        TimeLimit => "time_limit",
        /// The program ran out of memory on this test.
        MemoryLimit => "memory_limit",
        /// The program printed too much on this test.
        OutputLimit => "output_limit",
    }
}

text_enum! {
    /// The job state kept in the queue table.
    pub enum JobState {
        /// Waiting for a judge.
        Queued => "queued",
        /// A judge holds the lease.
        Claimed => "claimed",
        /// The run finished with a verdict.
        Succeeded => "succeeded",
        /// The run ended without a verdict.
        Failed => "failed",
        /// Retries ran out.
        Dead => "dead",
    }
}

text_enum! {
    /// Whether a run is a sample-only check or a hidden-test submission.
    pub enum RunMode {
        /// Runs the sample tests only.
        Run => "run",
        /// Runs the hidden tests.
        Submit => "submit",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn run_status_text_round_trips_with_hand_written_literals() {
        let cases = [
            (RunStatus::Queued, "queued"),
            (RunStatus::GhostHunting, "ghost_hunting"),
            (RunStatus::InternalError, "internal_error"),
        ];
        for (value, text) in cases {
            assert_eq!(value.as_str(), text);
            assert_eq!(RunStatus::parse(text), Some(value));
        }
    }

    #[test]
    fn run_status_has_exactly_nine_states_with_the_right_kinds() {
        assert_eq!(RunStatus::ALL.len(), 9);
        let terminal: Vec<&str> = RunStatus::ALL
            .iter()
            .filter(|s| s.is_terminal())
            .map(|s| s.as_str())
            .collect();
        assert_eq!(
            terminal,
            vec!["completed", "failed", "expired", "internal_error"]
        );
        let active: Vec<&str> = RunStatus::ALL
            .iter()
            .filter(|s| s.is_active())
            .map(|s| s.as_str())
            .collect();
        assert_eq!(
            active,
            vec!["claimed", "compiling", "running", "ghost_hunting"]
        );
    }

    #[test]
    fn verdict_text_matches_the_design_list() {
        let texts: Vec<&str> = Verdict::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "accepted",
                "wrong_answer",
                "runtime_error",
                "time_limit",
                "memory_limit",
                "output_limit",
                "compile_error",
            ]
        );
    }

    #[test]
    fn test_verdict_text_matches_the_design_list() {
        let texts: Vec<&str> = TestVerdict::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "pass",
                "wrong_answer",
                "runtime_error",
                "time_limit",
                "memory_limit",
                "output_limit",
            ]
        );
    }

    #[test]
    fn job_state_has_five_states() {
        let texts: Vec<&str> = JobState::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            texts,
            vec!["queued", "claimed", "succeeded", "failed", "dead"]
        );
    }

    #[test]
    fn run_mode_text_round_trips() {
        assert_eq!(RunMode::parse("run"), Some(RunMode::Run));
        assert_eq!(RunMode::parse("submit"), Some(RunMode::Submit));
        assert_eq!(RunMode::Submit.as_str(), "submit");
    }

    #[test]
    fn every_list_has_no_duplicates() {
        let statuses: HashSet<&str> = RunStatus::ALL.iter().map(|s| s.as_str()).collect();
        assert_eq!(statuses.len(), RunStatus::ALL.len());
        let verdicts: HashSet<&str> = Verdict::ALL.iter().map(|v| v.as_str()).collect();
        assert_eq!(verdicts.len(), Verdict::ALL.len());
    }

    #[test]
    fn invalid_text_is_rejected_case_sensitively() {
        assert_eq!(RunStatus::parse("Queued"), None);
        assert_eq!(RunStatus::parse(""), None);
        assert_eq!(Verdict::parse("accepted "), None);
    }

    #[test]
    fn serde_form_equals_the_text_form() {
        let json = serde_json::to_string(&RunStatus::GhostHunting).expect("serialise status");
        assert_eq!(json, "\"ghost_hunting\"");
        let back: RunStatus = serde_json::from_str("\"internal_error\"").expect("parse status");
        assert_eq!(back, RunStatus::InternalError);
        assert!(serde_json::from_str::<RunStatus>("\"bogus\"").is_err());
    }
}

text_enum! {
    /// Why a run ended without a verdict.
    pub enum FailureReason {
        /// The user asked for the run to stop.
        Cancelled => "cancelled",
        /// The problem is not published.
        ProblemUnavailable => "problem_unavailable",
        /// The language is not enabled.
        LanguageUnavailable => "language_unavailable",
        /// Waited in the queue too long.
        Expired => "expired",
        /// Every attempt failed for infrastructure reasons.
        RetriesExhausted => "retries_exhausted",
    }
}

impl FailureReason {
    /// A fixed, safe sentence for this reason. It never includes internal detail.
    #[must_use]
    pub const fn public_message(self) -> &'static str {
        match self {
            Self::Cancelled => "The run was cancelled before it produced a verdict.",
            Self::ProblemUnavailable => "This problem is not available right now.",
            Self::LanguageUnavailable => "This language is not available right now.",
            Self::Expired => "The run waited too long in the queue and was not run.",
            Self::RetriesExhausted => "The judge could not finish this run.",
        }
    }
}
