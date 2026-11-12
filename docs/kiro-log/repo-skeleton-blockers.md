# Repo skeleton blockers

## T10 — local Postgres migration verification (resolved)

The migration was initially implemented before local Postgres credentials were
available, so T10 stayed unchecked without reading `.env` or other secret
files. The Compose Postgres service later provided a controlled local database.
The migration was applied successfully, rerun successfully with no changes,
and an intentionally invalid migration exited non-zero. T10 is now complete.
