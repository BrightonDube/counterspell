# Driver log

Hourly entries from the Devin driver session(s). Format: time (SAST), spec,
tasks done/total, credits left, what was prompted, blockers.

---

## 2026-11-14 ~11:50 SAST — session start (Devin driver)

- **Kiro status:** not running. Only the `kiro-push` tmux session exists (pusher,
  untouched). Repo clean on `brighton/kiro-setup` @ `65c5564`; one old stash
  (`partial run-lifecycle T04 killed by WSL restart`) left as-is.
- **Credits:** 2978.08/4000 used → **~1022 remaining**, expires in 47 days
  (checked via `/usage` on the v2 engine). Above the 250 floor.
- **Task counts:** ghost-engine 37/37 done; grimoire-packs 26/30 (4 blocked);
  run-lifecycle 31/56; auth 20/55; judge-sandbox 0/56 (gated); language-drivers
  0/27 (gated); ide-workspace 13/39.
- **Checks run (all green):** `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -D warnings`, `cargo test --workspace` (62 binaries, 0
  failures), `cargo deny check licenses`, `gitleaks detect` (no leaks, 228
  commits), web `lint`/`check`/`test` (243)/`build`.
- **Test infra:** started throwaway `cs-test-pg` (Postgres 17) and
  `cs-test-valkey` (Valkey 8) in WSL Docker for the `pg_`/`valkey_` tests, on
  free host ports (check `docker ps` for the current mapping). BizPilot's
  containers were left untouched. Env vars for Kiro's next run:
  `DATABASE_URL` and `VALKEY_URL` pointing at those containers.
- **Prompted:** nothing — every unchecked priority task is blocked on a Brighton
  decision (see below). Re-prompting would burn credits for no progress.
- **Wrote:** `docs/reviews/judge-sandbox-T00.md` (T00 boundary review, ready for
  sign-off), `docs/reviews/run-lifecycle-2026-11-14.md` (commit review), this log.
- **Blockers needing Brighton:**
  1. **judge-sandbox T00 sign-off** — review doc ready; gates judge-sandbox,
     language-drivers, and downstream ide-workspace/grimoire T20 work.
  2. **run-lifecycle T29** — approve `futures-util` (preferred; already in
     Cargo.lock, MIT/Apache-2.0) or `futures-core` as a direct dep of
     `counterspell-server` + `THIRD-PARTY-NOTICES.md` entry. Unblocks T29–T31,
     T34–T37, T39.
  3. **auth T01** — choose: allow `CDLA-Permissive-2.0` in `deny.toml`, or switch
     `reqwest` to `rustls-tls-native-roots` (Kiro's assumed choice). Unblocks
     T14, T17–T23, T25, T27, T33–T37.
  4. **ide-workspace T05** — approve Geist + JetBrains Mono fonts
     (`THIRD-PARTY-NOTICES.md` entry).
  5. **grimoire-packs T28** — Brighton's personal review/publish of the three
     starter packs (human step, `status = "published"`).
  6. **grimoire-packs T29** — marked "blocked on ghost-engine" but ghost-engine
     is done; needs only confirmation of open question 11 (convert
     `packs/_test/` to the new format). Candidate for the next Kiro run once
     confirmed.
- **Next action:** on Brighton's decisions, write fix/continue prompts under
  `docs/kiro-prompts/devin/` and hand them to Kiro (one session at a time).

---

## 2026-11-14 ~12:05 SAST — review findings → Kiro fix run launched

- **Independent review:** spawned a second read-only reviewer over commits
  `bb612e5..65c5564`. It confirmed the primary review and found three verified
  defects the primary pass missed (all re-verified by driver against source):
  - **B1 blocker:** `docker-compose.yml` `server` env lacks `VALKEY_URL` →
    `docker compose up` crash-loops (`ConfigError::Missing`). T38's
    `docker compose config` verify couldn't catch it.
  - **M2 major:** `web/src/lib/runs/reducer.ts` maps `error` payloads by
    `code === 'internal_error'` — a code the server never sends; `expired` and
    `retries_exhausted` both display `failed`. Covering test uses the same
    impossible code (vacuous).
  - **M1 major (Brighton decision, pre-existing):** compose sets
    `POSTGRES_HOST_AUTH_METHOD: trust` — voids per-role least privilege on the
    internal network. Must be resolved (scram via env) before first real deploy.
  - Minors tracked in `docs/reviews/run-lifecycle-2026-11-14.md` (waker not yet
    wired — needed by T29/T30; `reconnects` never resets; migrate ordering;
    notify-after-commit edge).
- **DB suite:** `cargo test --workspace -- --include-ignored` with the throwaway
  Postgres/Valkey — 62 binaries ok, 0 failures. One earlier `VersionMissing`
  failure was a stale `sqlx::migrate!` embed on `/mnt/c`; fixed by
  `cargo clean -p counterspell-judge`. Not a code bug.
- **Prompted:** Kiro fix run launched in tmux `kiro-fix` via
  `docs/kiro-prompts/devin/run-review-fixes.sh` (prompt:
  `docs/kiro-prompts/devin/2026-11-14-review-fixes-compose-valkey-and-reducer.md`,
  mode `default`, model `claude-haiku-5.5`). Two fixes, one commit each, TDD.
  Note: `--mode implement` is invalid on kiro-cli 3.2.1 — use `default`.
- **Blockers unchanged:** judge-sandbox T00 sign-off (doc ready),
  `futures-util` approval (run-lifecycle T29), `reqwest`/`oauth2` licence
  (auth T01), fonts (ide-workspace T05), pack review (grimoire T28).

## 2026-11-14 ~12:20 SAST — PC crash recovery

- WSL restarted (uptime reset); all tmux sessions died. `kiro-fix` run was mid-Fix-1 boot smoke.
- Kiro's uncommitted work was GOOD and verified: `docker-compose.yml` VALKEY_URL line + `crates/server/tests/compose_env.rs` guard (passes). Fix 2 not started. Chose resume-over-stash to save credits.
- Test containers `cs-test-pg`/`cs-test-valkey` were Exited after crash; restarted, pg_isready + PONG ok.
- Relaunched Kiro in tmux `kiro-fix` with `docs/kiro-prompts/devin/2026-11-14-review-fixes-resume.md` (commit Fix 1, then Fix 2). Gotcha recorded: .sh launchers need LF endings — CRLF put `` in log filenames and killed the first two launches; `--mode implement` is invalid on kiro-cli 3.2.1, use `default`.
