---
inclusion: always
---
# TDD and code quality (implementation runs)

Workspace skills live in `.kiro/skills/` (sources and licences in `.kiro/skills/SOURCES.md`). Kiro loads them automatically. Use them by name:

| When | Skill |
|---|---|
| Before writing any production code for a task | `test-driven-development` (and its `writing-good-tests.md`) |
| Any failing test, build error, Clippy error or surprise | `systematic-debugging` |
| Before marking a task `[x]`, committing, or claiming anything passes | `verification-before-completion` |
| End of each wave and end of the spec | `requesting-code-review`, then `receiving-code-review` for the findings |
| Any `.svelte`, `.svelte.ts` or `.svelte.js` file | `svelte-core-bestpractices` |

Rules that apply on every task:

1. Inspect the existing code before changing it. Extend what exists; keep public APIs backward compatible; add no migration that a task does not require.
2. Red, green, refactor. Write the smallest failing test first, run it and see it fail for the expected reason, write the minimum code to pass, then refactor while green. Never write production code without a failing test. Exceptions: pure docs, config and asset files, which still get their task's named verification.
3. Expected values are hand-derived literals. Test real behaviour; mock only slow or external edges (network, clock, Docker). No test calls GitHub, Google, Resend or any external network. See "Test integrity" below.
4. Rust gate (for any Rust change): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Database tests follow the `pg_` / `valkey_` ignored-test convention and run against a throwaway local container when the task needs them.
5. Web gate (for any `web/` change): `cd web && npm run check && npm run build`, plus `npm run test` once a test script exists.
6. Runs are non-interactive. Where a skill or spec says "ask your human partner" or "ask Brighton", write the question and your assumed choice to `docs/kiro-log/<spec>-blockers.md` and continue with work that does not depend on the answer.
7. No browser or localhost end-to-end tests and no deploy environments for now (Brighton, 12 Nov 2026). Unit and integration tests only. Tasks that are only browser E2E (Playwright, screenshots, manual browser checks) stay unchecked and are listed as "deferred: E2E" in the blockers file.
8. Never read or write `.env` or any key or credential file. No `--privileged`. Never mount the main Docker socket into a container. Never run untrusted or hostile code on the host. Never push and never change branches.

## Test integrity
A passing suite only counts if the tests are real. These rules apply to every task, spec and hook-driven run.

**Never make a test pass by removing it or the code it covers.**
- Never delete, skip, `#[ignore]`, comment out or weaken a test because you cannot write good, clean code to make it pass. A failing test is information. Fix the code.
- Never delete or stub out a block of production code (a function, module, branch or error path) just so a test or the build goes green.
- Never loosen an assertion, widen an expected value, or special-case the test input to force a pass.
- If a test fails and you cannot fix it cleanly, stop and leave it failing. Write the test name, the failure, what you tried and your assumed next step to `docs/kiro-log/<spec>-blockers.md` (rule 6), then continue with work that does not depend on it. A red test with an honest note beats a green suite that hides a gap.

**When changing or removing a test is allowed.** Only when the requirement itself changed, or the test is provably wrong against the requirement. Name the requirement (for example `VAL-3`) in your summary, say why the old assertion was wrong, and do it in the same change. Add a comment `test-change: <REQ-ID>` in the edit, using an ID that exists in `docs/requirements.md` (for example `// test-change: VAL-3`). The guard hook accepts that marker and logs it to `docs/kiro-log/test-guard.log`.

**Write tests that check real behaviour.**
- Every test traces to a requirement or documented behaviour in `docs/requirements.md` or `docs/system-design.md`. It exercises a real action and asserts on the observable result.
- Never write mundane tests: a constant equal to itself, a struct holding the value you just put in it, getters and setters, "the function exists" or "returns without error", or a bare "is not empty" / "is Some".
- Never write a test that passes whatever the implementation does (no assertion, a mock that returns the expected value, or an expected value computed by the code under test).
- Use the public interface. Cover the failure paths and edge cases the requirement names, not only the happy path.

**Before claiming a task is done,** run the full gate for the language you changed and confirm the test count did not drop. If it dropped, say which tests were removed and why.

**Enforcement.** The `guard-test-integrity` hook (`scripts/kiro-hooks/guard-tests.sh`) blocks an edit that removes more tests than it adds in a source file, deletes a source file that contains tests, or adds a skip/ignore marker. It allows moving, renaming and refactoring tests, adding tests, `#[ignore]` on `pg_*` and `valkey_*` tests (rule 4), and any change carrying a valid `test-change:` marker. It cannot see edits made through the shell (`sed -i`, `rm`, `git checkout`), so do not use the shell to get around it. If it blocks you wrongly, use the marker if the requirement changed, otherwise log the blocker and move on.

## Global skills (~/.kiro/skills, linked from ~/.agents/skills)
Also use Brighton's global skills wherever relevant: tdd / pocock-tdd for every task, codebase-design and domain-modeling before new modules, diagnosing-bugs on any failure, code-review at each wave end, improve-codebase-architecture when refactoring, reliable-software-delivery for commits and checks, writing-for-agents for docs. If a global skill and a workspace skill overlap, follow both; the stricter rule wins. Never wait for interactive answers; log questions instead.
