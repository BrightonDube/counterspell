# Devin review fixes — two verified defects (14 Nov 2026)

Two defects were found reviewing the latest commits on `brighton/kiro-setup`.
Fix both, test-first where a test can express the defect, one commit per fix.
Follow strict TDD (red → green → refactor) and the workspace skills
(test-driven-development, verification-before-completion). Stay on
`brighton/kiro-setup`; never touch `main`. Do not weaken or delete existing
tests to make them pass.

## Fix 1 — `docker-compose.yml`: the `server` service is missing `VALKEY_URL` (blocker)

`crates/server/src/config.rs` requires `VALKEY_URL`
(`required(&lookup, "VALKEY_URL")`) and `crates/server/src/main.rs` opens the
`ValkeyLimiter` from it. The `server.environment` block in `docker-compose.yml`
sets `HOST`, `PORT`, `STATIC_DIR` and `DATABASE_URL` but no `VALKEY_URL`, so
`docker compose up` exits with `ConfigError::Missing("VALKEY_URL")` and
restart-loops. `docker compose config` cannot catch this — it only validates
YAML. T38's deploy notes (`deploy/run-lifecycle.md`) already list `VALKEY_URL`
in the config table, so the compose file is the side that is wrong.

Do:

- Add `VALKEY_URL` to the `server` service `environment:`, interpolated from
  the host/`--env-file` environment like `DATABASE_URL`. The compose `valkey`
  service has no auth and is only on the `internal` network, so
  `${VALKEY_URL:-redis://valkey:6379}` is the right default for this file.
- Check `deploy/vps/compose.prod.yml` and `deploy/vps/counterspell-deploy`:
  the production path runs `docker compose -f docker-compose.yml -f
  compose.prod.yml --env-file deploy.env`. The default above keeps prod working
  without new plumbing (same internal `valkey` service). Do NOT try to fix the
  separate gap that `deploy.env` carries no `SERVER_DATABASE_URL` /
  `JUDGE_DATABASE_URL` / `VALKEY_URL` — that is pre-deploy work owned by
  Brighton; just leave a one-line note in `docs/kiro-log/` recording that you
  saw it.
- Verify: `docker compose config` exits 0; then, if the images are already
  built or build is cheap, run `docker compose up -d server` far enough to see
  the server get past config validation (it may then fail on the empty
  `DATABASE_URL` — that is expected and fine). Record exactly what you
  observed in the commit message or `docs/kiro-log/`. If a boot smoke is not
  practical, say so honestly rather than claiming it.
- If you can add a cheap regression guard (for example a script or test that
  asserts every variable `ServerConfig` requires appears in the compose
  `server.environment` list), add it test-first. Otherwise the compose edit
  plus `docker compose config` is the verification; say which you did.

## Fix 2 — `web/src/lib/runs/reducer.ts`: the `error` → `status` mapping uses a code the server never sends (major)

`applyPayload`'s `error` case does
`status: payload.code === 'internal_error' ? 'internal_error' : 'failed'`.
`internal_error` is a `RunStatus`, not a `FailureReason` — the payload `code`
can never be `'internal_error'`. The real codes are in
`crates/common/src/status.rs` (`FailureReason::as_str`): `cancelled`,
`problem_unavailable`, `language_unavailable`, `expired`, `retries_exhausted`.
The Rust mapping in `crates/common/src/run_state.rs` is: `Expired → 'expired'`,
`RetriesExhausted → 'internal_error'`, the other three → `'failed'`.

Do (strict TDD):

- RED: in `web/src/lib/runs/reducer.test.ts`, add cases using ONLY real codes:
  `expired` → `status: 'expired'`; `retries_exhausted` →
  `status: 'internal_error'`; `cancelled`, `problem_unavailable` and
  `language_unavailable` → `status: 'failed'`. The existing test that uses
  `code: 'internal_error'` asserts an impossible payload — replace it with the
  real-code cases (or keep it only if it still asserts something true; it must
  not remain the only coverage of this branch).
- GREEN: fix the mapping in `reducer.ts` to match `run_state.rs` exactly.
- Verify: `cd web && npm run check && npm test && npm run build` — all clean.

## Rules

- One commit per fix; message style `fix(<scope>): <sentence case>` matching
  recent history (e.g. `fix(deploy): wire VALKEY_URL into the compose server`).
- Do not edit `.kiro/specs/**`, `tasks.md` checkboxes, `AGENTS.md`, or anything
  else outside the two fixes above, apart from the `docs/kiro-log/` note
  described in Fix 1.
- If either defect does not reproduce as described, stop and write what you
  found in `docs/kiro-log/` instead of forcing a change.
- Finish with a short summary: files changed, test names added, and the exact
  verification commands you ran with their results.
