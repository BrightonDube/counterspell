Implement the run-lifecycle spec in .kiro/specs/run-lifecycle/ (requirements.md, design.md, tasks.md). Follow AGENTS.md, every file in .kiro/steering/ (especially tdd-and-quality.md, security.md and rust-teaching.md), and the conventions at the top of tasks.md. Where design.md lists "Open questions", use the assumed choice unless Brighton has already answered it in the file.

This run is non-interactive and was approved by Brighton (12 Nov 2026). Work through the tasks in tasks.md in order, one task at a time. Skip `[stretch]` tasks this pass.

Skills: the workspace skills in .kiro/skills/ are loaded. Use them on every task:
- `test-driven-development` (read its writing-good-tests.md once at the start) before any production code;
- `systematic-debugging` for every failure;
- `verification-before-completion` before every tick and commit;
- `svelte-core-bestpractices` for any Svelte file;
- `requesting-code-review` at the end of each wave. If you can start a subagent, review with it using code-reviewer.md; otherwise do the review yourself against that checklist. Then use `receiving-code-review` to act on the findings (fix Critical and Important ones now, and list Minor ones in the hand-off).

For each task:
1. Read the task, its requirements and the code it touches. Inspect before you change anything; extend existing code and keep it backward compatible.
2. RED: write the smallest failing test (unit or integration) for the first behaviour. Run it and confirm it fails for the expected reason.
3. GREEN: write the minimum code that passes it. Repeat red/green until the task's behaviour and its named verification are covered.
4. REFACTOR while green: clear names, no duplication, no dead code, plain-English doc comments on public items.
5. Run the full relevant suite fresh. For Rust: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, plus the task's `pg_`/`valkey_` ignored tests when it touches the database. For web: `cd web && npm run check && npm run build`, plus `npm run test` when that script exists. Fix every failure with systematic-debugging before moving on.
6. Mark the task [x] in tasks.md only when the verification passed in this session. Append the task's Rust lesson to docs/kiro-log/rust-lessons.md when the task changed Rust.
7. Commit just that task: `git add` the files you changed (never `.env` or secrets), with a message like "run-lifecycle T03: <summary>". Git identity is already configured.

Scope limits:
- Unit and integration tests only. No browser or localhost end-to-end testing, no Playwright, and no deploy or staging environments. Leave E2E-only tasks unchecked as "deferred: E2E".
- Do not push and do not change branches. Never read or write .env files (use .env.example). Never run docker with --privileged, never mount the main Docker socket into any container, and never run untrusted code on the host.
- Add no crate, package or dependency that a task does not name. If `cargo deny check licenses` fails, stop that task and log it.
- `[SUPERVISED]` tasks: implement them test-first as normal, then append the task id, commit hash and what Brighton should check to docs/kiro-log/run-lifecycle-review-queue.md.
- Blockers: if a task needs something only Brighton can do, or a spec that is not implemented yet, leave it unchecked. Write the task id, the reason and what would unblock it to docs/kiro-log/run-lifecycle-blockers.md, then carry on with tasks that do not depend on it.

Spec notes: use the `UserResolver` and `ProblemLookup` seams; auth and the problem catalogue come later. The server must not depend on `bollard` or reach any Docker daemon. Use the judge `testkit` feature for end-to-end queue tests. Valkey holds counters only, never a queue. For `pg_` tests, start a throwaway `postgres:17-alpine` container with a throwaway password typed in the shell (never written to a file or committed), and stop it when done. Where a task reuses judge-sandbox vectors (T22), write the vectors the task needs locally and note the later reuse in the blockers file.

Finish with a summary table (task, status, tests added, verification result, commit hash), then the blockers, the review-queue items and the open Minor review findings.
