-- Run lifecycle guards (run-lifecycle T06). The transition table must equal
-- counterspell_common::run_state::can_transition. A test compares all 81 pairs (T09).

CREATE OR REPLACE FUNCTION cs_status_transition_allowed(from_status text, to_status text)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
AS $$
    SELECT CASE from_status
        WHEN 'queued'         THEN to_status IN ('queued', 'claimed', 'expired', 'failed')
        WHEN 'claimed'        THEN to_status IN ('claimed', 'compiling', 'queued', 'failed', 'internal_error')
        WHEN 'compiling'      THEN to_status IN ('compiling', 'running', 'completed', 'queued', 'failed', 'internal_error')
        WHEN 'running'        THEN to_status IN ('running', 'ghost_hunting', 'completed', 'queued', 'failed', 'internal_error')
        WHEN 'ghost_hunting'  THEN to_status IN ('ghost_hunting', 'completed', 'queued', 'failed', 'internal_error')
        WHEN 'completed'      THEN to_status = 'completed'
        WHEN 'failed'         THEN to_status = 'failed'
        WHEN 'expired'        THEN to_status = 'expired'
        WHEN 'internal_error' THEN to_status = 'internal_error'
        ELSE false
    END
$$;

CREATE OR REPLACE FUNCTION cs_submissions_guard()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- A finished run is frozen: none of the result columns may change.
    IF OLD.finished_at IS NOT NULL AND (
           NEW.status IS DISTINCT FROM OLD.status
        OR NEW.verdict IS DISTINCT FROM OLD.verdict
        OR NEW.failure_reason IS DISTINCT FROM OLD.failure_reason
        OR NEW.passed IS DISTINCT FROM OLD.passed
        OR NEW.total IS DISTINCT FROM OLD.total
        OR NEW.score IS DISTINCT FROM OLD.score
        OR NEW.runtime_ms IS DISTINCT FROM OLD.runtime_ms
        OR NEW.memory_kb IS DISTINCT FROM OLD.memory_kb
        OR NEW.started_at IS DISTINCT FROM OLD.started_at
        OR NEW.finished_at IS DISTINCT FROM OLD.finished_at
        OR NEW.user_id IS DISTINCT FROM OLD.user_id
        OR NEW.code_sha256 IS DISTINCT FROM OLD.code_sha256
        OR NEW.last_event_seq IS DISTINCT FROM OLD.last_event_seq
    ) THEN
        RAISE EXCEPTION 'finished run is frozen' USING ERRCODE = 'check_violation';
    END IF;

    IF NEW.status IS DISTINCT FROM OLD.status
       AND NOT cs_status_transition_allowed(OLD.status, NEW.status) THEN
        RAISE EXCEPTION 'status transition not allowed' USING ERRCODE = 'check_violation';
    END IF;

    IF NEW.last_event_seq < OLD.last_event_seq THEN
        RAISE EXCEPTION 'event sequence cannot decrease' USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS submissions_guard ON submissions;
CREATE TRIGGER submissions_guard
    BEFORE UPDATE ON submissions
    FOR EACH ROW
    EXECUTE FUNCTION cs_submissions_guard();
