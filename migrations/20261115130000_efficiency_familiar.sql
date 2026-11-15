-- Efficiency Familiar (efficiency-familiar T15). Adds the measured result columns,
-- the timing and baseline tables, their CHECK constraints, a freeze trigger and grants.
-- It depends on run-lifecycle (submissions, jobs, the two roles). If they are missing,
-- this migration fails loudly. Nothing here replaces run-lifecycle's freeze function.

ALTER TABLE submissions ADD COLUMN IF NOT EXISTS measured_complexity text;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS complexity_confidence text;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS fit jsonb;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS familiar_mood smallint;
ALTER TABLE submissions ADD COLUMN IF NOT EXISTS familiar_reason text;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'submissions_familiar_ck'
    ) THEN
        ALTER TABLE submissions ADD CONSTRAINT submissions_familiar_ck CHECK (
              (familiar_mood IS NULL OR familiar_mood BETWEEN 1 AND 5)
          AND (measured_complexity IS NULL OR measured_complexity IN ('1', 'log n', 'n', 'n log n', 'n^2', 'n^3', '2^n'))
          AND (complexity_confidence IS NULL OR complexity_confidence IN ('low', 'medium', 'high'))
          AND (measured_complexity IS NULL OR complexity_confidence IN ('medium', 'high'))
          AND (familiar_mood IS NULL OR status = 'completed')
          AND (familiar_mood IS NULL OR familiar_mood = 1 OR verdict = 'accepted')
          AND ((familiar_mood IS NULL) = (familiar_reason IS NULL))
          AND (familiar_reason IS NULL OR familiar_reason ~ '^[a-z_]{1,40}$')
          AND (fit IS NULL OR (jsonb_typeof(fit) = 'object' AND pg_column_size(fit) <= 4096))
        );
    END IF;
END
$$;

CREATE OR REPLACE FUNCTION cs_familiar_frozen()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- Once a run has finished, its measured result cannot change.
    IF OLD.finished_at IS NOT NULL AND (
           NEW.measured_complexity IS DISTINCT FROM OLD.measured_complexity
        OR NEW.complexity_confidence IS DISTINCT FROM OLD.complexity_confidence
        OR NEW.fit IS DISTINCT FROM OLD.fit
        OR NEW.familiar_mood IS DISTINCT FROM OLD.familiar_mood
        OR NEW.familiar_reason IS DISTINCT FROM OLD.familiar_reason
    ) THEN
        RAISE EXCEPTION 'familiar result is frozen' USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS cs_familiar_frozen ON submissions;
CREATE TRIGGER cs_familiar_frozen
    BEFORE UPDATE ON submissions
    FOR EACH ROW
    EXECUTE FUNCTION cs_familiar_frozen();

CREATE TABLE IF NOT EXISTS run_timings (
    submission_id uuid     NOT NULL REFERENCES submissions (id) ON DELETE CASCADE,
    attempt       smallint NOT NULL CHECK (attempt >= 1),
    n             integer  NOT NULL CHECK (n >= 1),
    repeat        smallint NOT NULL CHECK (repeat BETWEEN 1 AND 15),
    ns            bigint   NOT NULL CHECK (ns > 0),
    PRIMARY KEY (submission_id, attempt, n, repeat)
);

CREATE TABLE IF NOT EXISTS problem_baselines (
    id              bigserial PRIMARY KEY,
    problem_slug    text    NOT NULL,
    problem_version integer NOT NULL,
    lang            text    NOT NULL CHECK (lang IN ('python', 'javascript', 'java', 'cpp')),
    n               integer NOT NULL CHECK (n >= 1),
    median_ns       bigint  NOT NULL CHECK (median_ns > 0),
    image_digest    text    NOT NULL,
    engine_version  text    NOT NULL,
    fit_version     text    NOT NULL,
    measured_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (problem_slug, problem_version, lang, image_digest, n)
);

-- The server role inserts runs but must not write any familiar column. Table-level
-- INSERT would allow that, so it is replaced by a column list that leaves the five
-- familiar columns out.
REVOKE INSERT ON submissions FROM counterspell_server;
GRANT INSERT (id, user_id, problem_slug, lang, mode, code, code_sha256, idempotency_key,
              status, verdict, failure_reason, passed, total, score, runtime_ms, memory_kb,
              last_event_seq, created_at, started_at, finished_at)
    ON submissions TO counterspell_server;

-- Judge: writes the five columns in the same terminal UPDATE that sets finished_at.
GRANT UPDATE (measured_complexity, complexity_confidence, fit, familiar_mood, familiar_reason)
    ON submissions TO counterspell_judge;

REVOKE ALL ON run_timings, problem_baselines FROM PUBLIC;
GRANT SELECT, INSERT ON run_timings TO counterspell_judge;
GRANT SELECT, INSERT ON problem_baselines TO counterspell_judge;
GRANT USAGE ON SEQUENCE problem_baselines_id_seq TO counterspell_judge;
