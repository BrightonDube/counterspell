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
3. Expected values are hand-derived literals. Test real behaviour; mock only slow or external edges (network, clock, Docker). No test calls GitHub, Google, Resend or any external network.
4. Rust gate (for any Rust change): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. Database tests follow the `pg_` / `valkey_` ignored-test convention and run against a throwaway local container when the task needs them.
5. Web gate (for any `web/` change): `cd web && npm run check && npm run build`, plus `npm run test` once a test script exists.
6. Runs are non-interactive. Where a skill or spec says "ask your human partner" or "ask Brighton", write the question and your assumed choice to `docs/kiro-log/<spec>-blockers.md` and continue with work that does not depend on the answer.
7. No browser or localhost end-to-end tests and no deploy environments for now (Brighton, 12 Nov 2026). Unit and integration tests only. Tasks that are only browser E2E (Playwright, screenshots, manual browser checks) stay unchecked and are listed as "deferred: E2E" in the blockers file.
8. Never read or write `.env` or any key or credential file. No `--privileged`. Never mount the main Docker socket into a container. Never run untrusted or hostile code on the host. Never push and never change branches.

## Global skills (~/.kiro/skills, linked from ~/.agents/skills)
Also use Brighton's global skills wherever relevant: tdd / pocock-tdd for every task, codebase-design and domain-modeling before new modules, diagnosing-bugs on any failure, code-review at each wave end, improve-codebase-architecture when refactoring, reliable-software-delivery for commits and checks, writing-for-agents for docs. If a global skill and a workspace skill overlap, follow both; the stricter rule wins. Never wait for interactive answers; log questions instead.
