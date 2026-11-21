-- One application-owned FSRS card per learner and catalogue concept.
-- The composite primary key is both the uniqueness guard and the ownership key
-- used by concept_review_history.

CREATE TABLE IF NOT EXISTS concept_cards (
    user_id       uuid NOT NULL,
    concept_id    text NOT NULL,
    stability     double precision NOT NULL,
    difficulty    double precision NOT NULL,
    reps          integer NOT NULL DEFAULT 0,
    lapses        integer NOT NULL DEFAULT 0,
    state         text NOT NULL,
    due           timestamptz NOT NULL,
    last_review   timestamptz,
    version       bigint NOT NULL DEFAULT 0,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT concept_cards_pkey PRIMARY KEY (user_id, concept_id),
    CONSTRAINT concept_cards_user_fk
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT concept_cards_concept_fk
        FOREIGN KEY (concept_id) REFERENCES concepts (id) ON DELETE RESTRICT,
    CONSTRAINT concept_cards_stability_nonnegative CHECK (stability >= 0),
    CONSTRAINT concept_cards_difficulty_nonnegative CHECK (difficulty >= 0),
    CONSTRAINT concept_cards_reps_nonnegative CHECK (reps >= 0),
    CONSTRAINT concept_cards_lapses_nonnegative CHECK (lapses >= 0),
    CONSTRAINT concept_cards_version_nonnegative CHECK (version >= 0)
);

CREATE INDEX IF NOT EXISTS concept_cards_due
    ON concept_cards (user_id, due, concept_id);

REVOKE ALL ON concept_cards FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON concept_cards TO counterspell_server;
