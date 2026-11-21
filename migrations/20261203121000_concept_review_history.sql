-- Append-only audit trail for each accepted FSRS review.
-- A history row must point at the learner's existing card, so it cannot
-- describe a valid user/concept pair that has no scheduler state.

CREATE TABLE IF NOT EXISTS concept_review_history (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             uuid NOT NULL,
    concept_id          text NOT NULL,
    grade               text NOT NULL,
    reviewed_at         timestamptz NOT NULL,
    previous_card_state jsonb NOT NULL,
    next_card_state     jsonb NOT NULL,
    next_interval       interval NOT NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT concept_review_history_card_fk
        FOREIGN KEY (user_id, concept_id)
        REFERENCES concept_cards (user_id, concept_id) ON DELETE CASCADE,
    CONSTRAINT concept_review_history_grade_allowed
        CHECK (grade IN ('again', 'hard', 'good', 'easy')),
    CONSTRAINT concept_review_history_interval_nonnegative
        CHECK (next_interval >= interval '0 seconds')
);

CREATE INDEX IF NOT EXISTS concept_review_history_owner_reviewed
    ON concept_review_history (user_id, concept_id, reviewed_at DESC);
