# Repo skeleton blockers

## T10 — local Postgres migration verification

The migration file is implemented, but local migration execution is pending. The
available Postgres service requires credentials that are not present in the
workspace, and this task must not read `.env` or other secret files. Starting a
new long-running Docker database was not available through the current tool
control path. T10 remains unchecked until the Compose Postgres service can run
it during T14.
