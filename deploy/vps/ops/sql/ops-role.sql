-- Read-only operations login for the ops report (deploy-ops T17, R9.4). Brighton runs this once, as the
-- database owner, at T27. The password is set by hand in psql with \password cs_ops_ro. It is never
-- written into this file or the repository.
--
-- The grants are column-level SELECT only, on the columns the ops report reads. Nothing else: no code,
-- no payload, no output and no hidden test data.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cs_ops_ro') THEN
        CREATE ROLE cs_ops_ro LOGIN NOINHERIT;
    END IF;
END
$$;

GRANT USAGE ON SCHEMA public TO cs_ops_ro;

GRANT SELECT (state, run_after, lease_expires_at, updated_at) ON jobs TO cs_ops_ro;
GRANT SELECT (id, status, created_at, finished_at) ON submissions TO cs_ops_ro;
