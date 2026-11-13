-- Auth privileges (auth T08). The roles are created by the run-lifecycle migration; this file
-- only makes sure they exist (so it can run on its own) and then sets the matrix from
-- .kiro/specs/auth/design.md. The judge role gets nothing on auth tables. The server role gets
-- only the columns it needs. Columns that identify the demo account are never updatable here.

DO $$
BEGIN
    CREATE ROLE counterspell_server NOLOGIN;
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

REVOKE ALL ON users, identities, email_tokens, sessions, oauth_flows FROM PUBLIC;
REVOKE ALL ON users, identities, email_tokens, sessions, oauth_flows FROM counterspell_judge;

-- Server: signs people in, creates and deletes accounts, and manages its own tokens and sessions.
GRANT SELECT, INSERT, DELETE ON users TO counterspell_server;
GRANT UPDATE (email, email_verified_at, password_hash, display_name, country, xp, streak_days,
              last_active, familiar_prefs, updated_at)
    ON users TO counterspell_server;

GRANT SELECT, INSERT ON identities TO counterspell_server;

GRANT SELECT, INSERT, DELETE ON email_tokens TO counterspell_server;
GRANT UPDATE (used_at) ON email_tokens TO counterspell_server;

GRANT SELECT, INSERT, DELETE ON sessions TO counterspell_server;
GRANT UPDATE (last_renewed_at, expires_at) ON sessions TO counterspell_server;

GRANT SELECT, INSERT, DELETE ON oauth_flows TO counterspell_server;
