-- Run lifecycle tables (run-lifecycle T05). The Rust code in counterspell-common
-- relies on these names and CHECK values, so change them only with the design.
-- Foreign keys to users and problems are added later by the auth and catalogue specs.

CREATE TABLE IF NOT EXISTS submissions (
    id              uuid PRIMARY KEY,
    user_id         uuid NOT NULL,
    problem_slug    text NOT NULL,
    lang            text NOT NULL CHECK (lang IN ('python', 'javascript', 'java', 'cpp')),
    mode            text NOT NULL CHECK (mode IN ('run', 'submit')),
    code            text NOT NULL CHECK (octet_length(code) <= 262144),
    code_sha256     bytea NOT NULL,
    idempotency_key text,
    status          text NOT NULL DEFAULT 'queued' CHECK (status IN (
                        'queued', 'claimed', 'compiling', 'running', 'ghost_hunting',
                        'completed', 'failed', 'expired', 'internal_error')),
    verdict         text CHECK (verdict IN (
                        'accepted', 'wrong_answer', 'runtime_error', 'time_limit',
                        'memory_limit', 'output_limit', 'compile_error')),
    failure_reason  text CHECK (failure_reason IN (
                        'cancelled', 'problem_unavailable', 'language_unavailable',
                        'expired', 'retries_exhausted')),
    passed          int,
    total           int,
    score           int,
    runtime_ms      int,
    memory_kb       int,
    last_event_seq  int NOT NULL DEFAULT 0 CHECK (last_event_seq >= 0),
    created_at      timestamptz NOT NULL DEFAULT now(),
    started_at      timestamptz,
    finished_at     timestamptz,
    -- A finished run has finished_at. Only a completed run has a verdict.
    -- Only the three non-verdict endings carry a failure reason.
    CHECK ((status IN ('completed', 'failed', 'expired', 'internal_error')) = (finished_at IS NOT NULL)),
    CHECK ((status = 'completed') = (verdict IS NOT NULL)),
    CHECK ((status IN ('failed', 'expired', 'internal_error')) = (failure_reason IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS submissions_user_created_idx
    ON submissions (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS submissions_user_problem_idx
    ON submissions (user_id, problem_slug, created_at DESC);
CREATE UNIQUE INDEX IF NOT EXISTS submissions_user_idempotency_idx
    ON submissions (user_id, idempotency_key) WHERE idempotency_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS submissions_user_active_idx
    ON submissions (user_id) WHERE finished_at IS NULL;

CREATE TABLE IF NOT EXISTS jobs (
    id                   bigserial PRIMARY KEY,
    submission_id        uuid NOT NULL UNIQUE REFERENCES submissions (id) ON DELETE CASCADE,
    state                text NOT NULL DEFAULT 'queued'
                             CHECK (state IN ('queued', 'claimed', 'succeeded', 'failed', 'dead')),
    attempts             smallint NOT NULL DEFAULT 0,
    max_attempts         smallint NOT NULL DEFAULT 3,
    lease_owner          text,
    lease_expires_at     timestamptz,
    claimed_at           timestamptz,
    run_after            timestamptz NOT NULL DEFAULT now(),
    cancel_requested_at  timestamptz,
    last_error           text CHECK (char_length(last_error) <= 500),
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    CHECK (max_attempts >= 1),
    CHECK (attempts >= 0 AND attempts <= max_attempts),
    CHECK (state <> 'claimed' OR (lease_owner IS NOT NULL AND lease_expires_at IS NOT NULL)),
    CHECK (state <> 'queued' OR lease_owner IS NULL)
);

CREATE INDEX IF NOT EXISTS jobs_queue_idx
    ON jobs (run_after, id) WHERE state = 'queued';
CREATE INDEX IF NOT EXISTS jobs_lease_idx
    ON jobs (lease_expires_at) WHERE state = 'claimed';

CREATE TABLE IF NOT EXISTS run_events (
    submission_id uuid NOT NULL REFERENCES submissions (id) ON DELETE CASCADE,
    seq           int NOT NULL CHECK (seq >= 1),
    attempt       smallint NOT NULL CHECK (attempt >= 1),
    type          text NOT NULL CHECK (type IN (
                      'compiling', 'test', 'ghost_hunting', 'shrink_step', 'ghost',
                      'bench_point', 'complexity', 'familiar', 'restarted', 'done', 'error')),
    payload       jsonb NOT NULL CHECK (
                      jsonb_typeof(payload) = 'object' AND pg_column_size(payload) <= 16384),
    created_at    timestamptz NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (submission_id, seq)
);

-- At most one terminal event (done or error) per run.
CREATE UNIQUE INDEX IF NOT EXISTS run_events_one_terminal
    ON run_events (submission_id) WHERE type IN ('done', 'error');

CREATE TABLE IF NOT EXISTS test_results (
    submission_id uuid NOT NULL REFERENCES submissions (id) ON DELETE CASCADE,
    ordinal       int NOT NULL CHECK (ordinal >= 1),
    is_sample     boolean NOT NULL,
    verdict       text NOT NULL CHECK (verdict IN (
                      'pass', 'wrong_answer', 'runtime_error', 'time_limit',
                      'memory_limit', 'output_limit')),
    time_ms       int NOT NULL CHECK (time_ms >= 0),
    memory_kb     int,
    public_detail jsonb,
    stdout_snip   text,
    stderr_snip   text,
    PRIMARY KEY (submission_id, ordinal),
    -- Hidden tests never store input, expected output or program output.
    CHECK (is_sample OR (public_detail IS NULL AND stdout_snip IS NULL AND stderr_snip IS NULL))
);
