# Hints spec blockers and open decisions

Run: 16 Nov 2026 implement run (non-interactive). Each entry names the task, the reason, and what would unblock it.

## G2: authored hint length (decision needed from Brighton)

- The hints spec (G2, R1.1 and the migration design) says each rung is 1 to 1,200 characters.
- The database CHECK in `migrations/20261116100000_catalogue_core.sql` (`problem_hints.body`) and the grimoire-packs validator (`hints.toml` rule HNT-02, tested in `crates/grimoire/src/hints.rs`) both say 1 to 600 characters.
- The stricter 600 limit is kept, so no stored hint can break the database check. G2 stays open until Brighton picks one value. If 1,200 wins, the catalogue CHECK, the grimoire limit and the grimoire tests all change together, with a `test-change:` marker that cites the requirement.
- G2 also asks for errors that name the problem slug and the rung. The current messages name the file and the JSON pointer, not the slug. This is left open with G2.

## H11 / ghost read: `ghosts` table has no owner

- The hints store reads the submission's ghost (R3.1). No migration creates a per-submission `ghosts` table yet. The catalogue spec lists the owner as unresolved (decision D16, `.kiro/specs/problem-catalogue/tasks.md`).
- Unblock: decide the owner of `ghosts`. Until then the ghost read returns no ghost when the table is missing, so AI hints still work on the verdict and code (R3.5).

## Run 16 Nov 2026 (attempt 2): status of the open items

### G2 still open: length decision (Brighton)

- Unchanged from above. The spec says 1 to 1,200 characters. The database CHECK and the grimoire rule say 1 to 600. The stricter 600 limit is still enforced everywhere, so G2 stays unchecked. The grimoire errors also name the file and the JSON pointer, not the problem slug (G2 asks for the slug).
- Unblock: Brighton picks 600 or 1,200. Then the CHECK, the grimoire limit and the grimoire tests change together.

### H-ci: CI secret step not added (proof needs a scratch branch)

- `.github/workflows/security.yml` already runs gitleaks over the whole repository on every push and PR (`gitleaks git . --config .gitleaks.toml`). It already catches OpenAI and Groq style keys that a file might contain, so the spec's "one added step" would duplicate it. The P9 log test (`a_failed_call_logs_no_key_and_no_prompt_text`) runs in `cargo test`.
- The spec's proof (a fake key on a scratch branch, failing then passing) needs a branch, and this run may not change branches. Left unchecked.
- Unblock: Brighton either accepts the existing gitleaks job as the H-ci check (then H-ci can be ticked as "covered by security.yml"), or allows a scratch-branch proof.

### H-verify: manual smoke test deferred (E2E)

- All automated checks passed in this session: fmt, clippy (workspace, all targets), `cargo test --workspace` (1,307 passed, 0 failed), `cargo deny check licenses` (ok), the ignored `pg_`/`valkey_` hints tests, `npm run test` (64 files), `npm run check`, `npm run build`, and `docker compose config` (pass-throughs present, Valkey append-only on).
- The manual smoke test (sign in with the demo account, reveal rung 1, one AI hint, check the label, notice, quota and logs) is browser work. It is deferred: E2E. Checklist and status are in `docs/kiro-log/hints-smoke.md`.

### ide-workspace adapter out of contract (not used)

- `web/src/lib/api/hints.ts` (ide-workspace T17) expects `{ rung, text }` from `POST /api/hints`. The server returns `{ rung, source, body, label, ... }`, so that adapter would fail on a real reply. Nothing imports `requestAuthoredHint`, so it is dead code today.
- H15 added the contract-correct client in `web/src/lib/hints/client.ts`, as the spec asks. The old file is left in place, because it belongs to ide-workspace.
- Unblock: the ide-workspace owner removes `web/src/lib/api/hints.ts` and its test, or points the panel at `web/src/lib/hints/`.

### HTTP-level fallback and provider tests use the pipeline, not the router

- `HintState` holds a concrete `ReqwestTransport`, so an HTTP test cannot script provider replies. Fallback, success and failure of the AI pipeline are tested with `ScriptedTransport` in `hints::ai` (unit and `valkey_` tests). The HTTP tests cover the cap refusal, the ladder and the cache hit.
- Decision to make later: making `HintState` generic over the transport would let HTTP tests cover fallback end to end. Not done here, because it changes the router's public type.

## Run 16 Nov 2026 (attempt 3): no change in status

- G2: still open. The spec says 1 to 1,200 characters. The migration CHECK and the grimoire rule say 1 to 600. Brighton needs to pick one value (see the G2 entry above). This is a product choice, not a task that code can settle.
- H-ci: still open. `.github/workflows/security.yml` already runs gitleaks over full history and TruffleHog on every push and PR, so the spec's extra step would duplicate them. The spec's proof (a fake key on a scratch branch) needs a branch, and this run may not change branches. Brighton can accept the existing jobs as the H-ci check, or allow a scratch-branch proof.
- H-verify: all automated checks pass again on HEAD c0f0612 (see `docs/kiro-log/hints-smoke.md`). The manual smoke test is browser work and stays deferred: E2E.

## drive queue run (kiro/drive-queue): G2 done, length decision recorded
- **Decision (routine, under the standing "decide and record" rule; not a security, licence, cost or production-data call; Brighton may overrule):** keep the 600-character hint limit. It is stricter than the spec's 1,200, so every hint the spec accepts as "at most 1,200" is still bounded, it matches the shipped `problem_hints` CHECK, and it needs no migration. If Brighton wants 1,200, the CHECK, `MAX_HINT_CHARACTERS` and the HNT-02 tests change together.
- G2 delivered in `crates/grimoire/src/hints.rs`: messages now say `rung N` (the pack name is already in every report and in the JSON `pack` field), and an indented code block longer than 3 lines is rejected like a fenced one. Tests: `a_too_long_hint_names_its_rung`, `a_long_code_fence_names_its_rung`, `indented_code_block_of_four_lines_is_hnt_02`, plus allowed cases for a 3-line indented block and two indented prose lines. The three failing ones were red before the change. Workspace: 1573 passed, 0 failed; fmt and clippy clean.
- Still open: H-ci (the existing gitleaks and TruffleHog jobs already cover it; waits on Brighton accepting that or allowing a scratch-branch proof) and H-verify (manual browser smoke test, now in scope since Playwright was approved; needs the hints UI on the test server).
