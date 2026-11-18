-- Custom ghost runs (ide-workspace T02). A ghost run is a `run` submission that
-- replays the user's code on one stored ghost input instead of the sample tests.
-- `ghost_input` is the single custom input; `ghost_source_id` is provenance: the
-- earlier submission the ghost came from. The judge executes the custom input and
-- compares it against the trusted reference, never against user-declared data.

ALTER TABLE submissions ADD COLUMN IF NOT EXISTS ghost_input jsonb;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS ghost_source_id uuid
    REFERENCES submissions (id) ON DELETE SET NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'submissions_ghost_ck'
    ) THEN
        ALTER TABLE submissions ADD CONSTRAINT submissions_ghost_ck CHECK (
              -- A source id without an input is meaningless. An input alone is the
              -- state a row reaches when its source submission is deleted.
              (ghost_source_id IS NULL OR ghost_input IS NOT NULL)
          AND (ghost_input IS NULL OR pg_column_size(ghost_input) <= 65536)
        );
    END IF;
END
$$;

-- The server role inserts runs but must not write any familiar column, so its
-- INSERT is a column list. Extend it with the two ghost columns (the list from
-- efficiency_familiar plus ghost_input and ghost_source_id).
REVOKE INSERT ON submissions FROM counterspell_server;
GRANT INSERT (id, user_id, problem_slug, lang, mode, code, code_sha256, idempotency_key,
              status, verdict, failure_reason, passed, total, score, runtime_ms, memory_kb,
              last_event_seq, created_at, started_at, finished_at,
              ghost_input, ghost_source_id)
    ON submissions TO counterspell_server;
