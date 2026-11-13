-- Run lifecycle roles (run-lifecycle T07). Roles are cluster-wide, so creation
-- ignores duplicates. They have no passwords and cannot log in. Deployment creates
-- login users and grants membership (deploy/run-lifecycle.md). Passwords never live here.
--
-- The server role cannot forge events, results or verdicts. The judge role cannot
-- change who owns a run or what code it contains.

DO $$
BEGIN
    CREATE ROLE counterspell_server NOLOGIN;
-- Two databases migrating at once can race on the cluster-wide role catalogue.
-- The loser gets unique_violation rather than duplicate_object, so catch both.
EXCEPTION WHEN duplicate_object OR unique_violation THEN
    NULL;
END
$$;

DO $$
BEGIN
    CREATE ROLE counterspell_judge NOLOGIN;
EXCEPTION WHEN duplicate_object OR unique_violation THEN
    NULL;
END
$$;

REVOKE ALL ON submissions, jobs, run_events, test_results FROM PUBLIC;

-- Server: creates runs and their jobs, reads everything, and asks for cancellation.
GRANT SELECT, INSERT ON submissions TO counterspell_server;
GRANT SELECT, INSERT ON jobs TO counterspell_server;
GRANT UPDATE (cancel_requested_at, updated_at) ON jobs TO counterspell_server;
GRANT USAGE ON SEQUENCE jobs_id_seq TO counterspell_server;
GRANT SELECT ON run_events, test_results TO counterspell_server;

-- Judge: claims jobs and writes results. It cannot delete or change ownership or code.
GRANT SELECT ON submissions, jobs, run_events, test_results TO counterspell_judge;
GRANT UPDATE (status, verdict, failure_reason, passed, total, score, runtime_ms, memory_kb,
              last_event_seq, started_at, finished_at)
    ON submissions TO counterspell_judge;
GRANT UPDATE (state, attempts, lease_owner, lease_expires_at, claimed_at, run_after,
              last_error, updated_at)
    ON jobs TO counterspell_judge;
GRANT INSERT ON run_events, test_results TO counterspell_judge;
