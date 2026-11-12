# Tasks: counterspell-mcp-power

All tasks below are implementation tasks for the design above. No task is being executed by this spec-generation run. Each task is intended to take 30 minutes or less. `[P]` means it may run in parallel with other tasks in the same wave after its dependencies complete. `[S]` means it is sequential or has a required dependency.

## Wave 0: decisions and contracts

- [ ] **T01 [S] (20 min) Confirm external schemas and dependency versions**
  - Files: `.kiro/specs/counterspell-mcp-power/design.md` only if decisions need recording; later `Cargo.toml`/lockfile during implementation.
  - Verify the official `rmcp` release/features, its stdio server API, the Agent Plugins `plugin.json`/`mcp.json` schema, and the supported Power working-directory syntax.
  - Preserve: six-package workspace, official Rust SDK, Apache-2.0-compatible licensing, and the exact six tool names.
  - Depends on: none.
  - Verify: selected versions and any schema spelling changes are recorded before code is written.

- [ ] **T02 [S] (25 min) Define the shared typed runner contract**
  - Files: existing `crates/judge/` library boundary and any engine/Grimoire interface adjustments required by their approved specs.
  - Define the trusted runner request, target, bounded result, infrastructure failure, and judge-backed Oracle/PackRunner adapter. Keep expected outputs, comparator, deadlines, and cleanup in the judge.
  - Preserve: MCP may depend on the existing judge package library, but no seventh crate or host execution path is introduced.
  - Depends on: T01 and the completed engine, Grimoire, and judge-sandbox contracts.
  - Verify: a fake runner can be injected in tests; production construction requires the sandbox-only endpoint.

## Wave 1: pure engine and Grimoire seams

- [ ] **T03 [P] (30 min) Expose or complete the engine adapter surface**
  - Files: `crates/engine/**` as required by the ghost-engine spec.
  - Ensure MCP can call the public schema, comparator, hunt, shrink, autopsy/difference, provenance, result serialization, and pure complexity-fit APIs without duplicate algorithms.
  - Preserve: engine remains synchronous, pure, and free of process, filesystem, Docker, network, Tokio, and HTTP dependencies.
  - Depends on: T02 and the approved engine implementation tasks.
  - Verify: engine property tests and JSON round-trip tests pass.

- [ ] **T04 [P] (30 min) Expose the Grimoire static validator library contract**
  - Files: `crates/grimoire/**` as required by the Grimoire spec.
  - Make the MCP call `validate_static`/equivalent typed library APIs and return the existing diagnostic report. Keep static validation execution-free and avoid shelling out to the Grimoire binary.
  - Preserve: hidden data is not printed; sandbox validation remains a separate judge-backed operation.
  - Depends on: T02 and the approved Grimoire implementation tasks.
  - Verify: static reports and diagnostic codes match the Grimoire CLI contract.

## Wave 2: judge runner library and boundary

- [ ] **T05 [S] (30 min) Add a reusable library target to `counterspell-judge`**
  - Files: `crates/judge/Cargo.toml`, `crates/judge/src/lib.rs`, `crates/judge/src/runner/**`, `crates/judge/src/main.rs` only as needed.
  - Share the production sandbox runner between the worker binary and MCP. Keep the sandbox Docker client in judge code and connect only to the separate rootless sandbox daemon.
  - Preserve: no main Docker socket, no public port, no host interpreter, no Docker CLI, and no weakening of production limits.
  - Depends on: T02, T03, T04, and the judge-sandbox implementation.
  - Verify: worker and a library consumer use the same runner configuration, images, driver, comparator, deadlines, bounded output, and cleanup.

- [ ] **T06 [S] (25 min) Add the MCP-facing trusted Oracle/PackRunner adapter**
  - Files: `crates/judge/src/runner/**` and `crates/mcp/src/judge_adapter.rs`.
  - Adapt judge execution into the engine's existing synchronous observation/probe contract without adding runtime or Docker dependencies to the engine.
  - Preserve: infrastructure errors remain infrastructure errors; user crashes/timeouts are classified only by the trusted judge/engine contract.
  - Depends on: T05.
  - Verify: fake-runner tests cover Found, NotFound, Aborted, crash, timeout, garbled output, and reference failure cases.

## Wave 3: MCP protocol and tools

- [ ] **T07 [S] (25 min) Add MCP dependencies and stdio service startup**
  - Files: `crates/mcp/Cargo.toml`, `crates/mcp/src/main.rs`, `crates/mcp/src/server.rs`, `crates/mcp/src/config.rs`.
  - Add the pinned official `rmcp` dependency and required serde/Tokio/path dependencies. Start a stdio server with protocol output on stdout, logs on stderr, clean EOF/cancellation, and no HTTP listener.
  - Preserve: no `unwrap()`/`expect()` in production Rust; missing sandbox config must not prevent pure-tool registration unless the SDK requires a single initialization path.
  - Depends on: T01, T05, and T06.
  - Verify: an MCP initialize/list-tools smoke test succeeds and no socket listener appears.

- [ ] **T08 [P] (25 min) Implement tool schemas and safe error mapping**
  - Files: `crates/mcp/src/schemas.rs`, `crates/mcp/src/errors.rs`, `crates/mcp/src/paths.rs`.
  - Define bounded request/response types, stable error categories, workspace-relative path validation, symlink/traversal rejection, and redaction/truncation helpers.
  - Preserve: no absolute host path disclosure, hidden data disclosure, secret reads, or unbounded output.
  - Depends on: T01 and T06.
  - Verify: malformed, oversized, unknown-field, invalid-path, invalid-budget, and missing-sandbox cases return structured errors without panics.

- [ ] **T09 [P] (30 min) Implement pure/read-only tools**
  - Files: `crates/mcp/src/tools.rs`, engine/Grimoire adapter modules as needed.
  - Implement `validate_pack`, `explain_verdict`, and `fit_complexity` using existing libraries. Keep `validate_pack` static-only and `fit_complexity` pure timing fitting.
  - Preserve: no code execution, no LLM call, no repository writes, no duplicate validator/autopsy/fit logic.
  - Depends on: T03, T04, T07, and T08.
  - Verify: tool outputs match Grimoire/engine reports and can be safely auto-approved.

- [ ] **T10 [S] (30 min) Implement code-running tools through the judge**
  - Files: `crates/mcp/src/tools.rs`, `crates/mcp/src/judge_adapter.rs`, judge runner integration only as needed.
  - Implement `run_reference`, `find_ghost_input`, and `shrink_input` with explicit sandbox-backed descriptions, production-compatible budgets, trusted comparator, cleanup, and error mapping.
  - Preserve: no host Python/Node/shell, no direct Docker access from MCP, same production images/limits, and no auto-approval.
  - Depends on: T06, T07, T08, and T09.
  - Verify: fake-runner protocol tests pass; real rootless sandbox smoke test proves code runs only in the sandbox and infrastructure failure is not Wrong Answer.

## Wave 4: Power packaging and workspace registration

- [ ] **T11 [P] (25 min) Create the Agent Plugins Power manifest**
  - Files: `kiro-power/plugin.json`, `kiro-power/mcp.json`.
  - Add the Counterspell identity, description, required keywords, repository-relative stdio command, tool registration, and the exact pure-tool auto-approval list.
  - Preserve: no machine-specific absolute paths, credentials, or auto-approved execution tools.
  - Depends on: T01 and T07.
  - Verify: clean-checkout JSON/schema validation and Kiro custom-Power import test.

- [ ] **T12 [P] (25 min) Write the problem-authoring skill**
  - Files: `kiro-power/skills/author-problem/SKILL.md`.
  - Document original-content rules, Grimoire layout, schema/generator/reference/hints, static-first validation, explicit approval for execution, and sandbox-only code execution.
  - Preserve: no host execution instructions and no copied third-party problem text.
  - Depends on: T04 and T11.
  - Verify: skill references the six canonical tools accurately and is usable from a clean checkout.

- [ ] **T13 [P] (25 min) Write the ghost-hunting skill**
  - Files: `kiro-power/skills/hunt-ghost/SKILL.md`.
  - Document reproduce → find → inspect → explain → fix → validate → re-hunt, honest minimized-counterexample language, and explicit approval for code-running tools.
  - Preserve: never claim global minimality and never instruct Kiro to run pack code directly.
  - Depends on: T10 and T11.
  - Verify: a reviewer can follow the skill without inventing an unsupported tool or execution path.

- [ ] **T14 [S] (20 min) Register the workspace MCP server**
  - Files: `.kiro/settings/mcp.json`.
  - Register `counterspell-mcp` with the same command and working-directory strategy as the Power. Auto-approve only `validate_pack`, `explain_verdict`, and `fit_complexity`.
  - Preserve: no secrets and no execution tool in `autoApprove`.
  - Depends on: T11 and T13.
  - Verify: parse the configuration and compare its server/tool/allow-list values with `kiro-power/mcp.json`.

## Wave 5: dogfood pack and evidence

- [ ] **T15 [P] (30 min) Create the private dogfood pack**
  - Files: `packs/_demo/mcp-dogfood/**` and pack tests/fixtures only where the approved Grimoire layout requires them.
  - Add original problem text, metadata with `dogfood = true`, correct reference, deliberately buggy implementation, hints, and optional known ghost. Keep `packs/_test/` unchanged and ensure public import skips this directory.
  - Preserve: no host execution of the Python files; no hidden data in the pack or diagnostics.
  - Depends on: T04 and T12.
  - Verify: static `validate_pack` passes after the intentional implementation bug is present.

- [ ] **T16 [S] (30 min) Run the scripted Kiro find/fix/re-verify session**
  - Files: the dogfood implementation under the pack; later evidence paths under `docs/`.
  - Have Kiro explicitly call `find_ghost_input`, inspect the returned minimized counterexample/shrink trace, fix the buggy implementation, call `validate_pack`, and call `find_ghost_input` again.
  - Preserve: all execution goes through the sandbox runner; the session states the bounded/minimized meaning honestly.
  - Depends on: T10, T14, and T15.
  - Verify: first hunt finds the intended failure; validation passes after the fix; second hunt returns no counterexample within budget or the approved stronger verified-pass result.

- [ ] **T17 [S] (20 min) Save scrubbed dogfood evidence**
  - Files: `docs/kiro-log/chats/counterspell-mcp-dogfood.*`, `docs/media/kiro/mcp-dogfood-*.png`, and `docs/KIRO_USAGE.md`.
  - Export the chat, capture the MCP call/shrink result/fix/re-verification/Power install screenshots, scrub secrets and hidden data, and add factual links and claims to the usage document.
  - Preserve: evidence must show actual behavior, not fabricated results; do not commit credentials or private values.
  - Depends on: T16 and the Power import test in T11.
  - Verify: artifacts exist at the documented paths, are readable, contain no secrets, and support the video sequence.

## Wave 6: final verification

- [ ] **T18 [S] (25 min) Validate package/config boundaries**
  - Files: tests and validation scripts in the relevant existing packages only.
  - Run Power manifest checks, MCP settings checks, path/security scans, dependency/licence checks, and verify that no execution tool is auto-approved.
  - Preserve: exactly six Cargo packages and no main Docker socket in MCP/server configuration.
  - Depends on: T09, T10, T11, T12, T13, T14, and T17.
  - Verify: all static checks pass and every security invariant in requirements is covered by a test or documented target-host check.

- [ ] **T19 [S] (30 min) Run the complete MCP and sandbox verification**
  - Files: only implementation fixes required by failing checks; no unrequested refactors.
  - Run fmt, Clippy with warnings denied, workspace/package tests, MCP protocol tests, fake-runner tests, Power/config tests, static pack validation, and the real sandbox smoke test with the separate rootless daemon.
  - Preserve: infrastructure errors remain distinct, no host fallback exists, and the sandbox daemon is not the main Compose daemon.
  - Depends on: T18.
  - Verify: every required check passes; record any environment-dependent rootless/cgroup verification honestly.

## Dependency summary and parallel waves

- **Wave 0:** T01 is the external-schema gate. T02 follows it and the existing engine/Grimoire/judge contracts.
- **Wave 1:** T03 and T04 can run in parallel after T02.
- **Wave 2:** T05 is sequential because it defines the trusted runner library. T06 follows T05.
- **Wave 3:** T07 follows the dependency/API decisions and runner seam. T08 and T09 can run in parallel after their listed dependencies. T10 follows the runner and protocol scaffolding.
- **Wave 4:** T11, T12, and T13 can proceed after their listed inputs. T14 follows the two configuration sources.
- **Wave 5:** T15 can proceed after Grimoire and the authoring skill. T16 requires the real tools, workspace registration, and pack. T17 follows the observed session.
- **Wave 6:** T18 and T19 are sequential final gates.

## Completion checklist

- [ ] `counterspell-mcp` runs as an official `rmcp` stdio server with six registered tools.
- [ ] Pure tools reuse engine/Grimoire logic and code-running tools use the judge runner.
- [ ] No MCP path executes Python/Node/shell on the developer host or uses the main Docker daemon.
- [ ] Code-running tools have explicit sandbox descriptions and are absent from auto-approval lists.
- [ ] `validate_pack`, `explain_verdict`, and `fit_complexity` are the only auto-approved tools.
- [ ] `kiro-power/` contains the required Agent Plugins manifest, MCP registration, and both skills.
- [ ] `.kiro/settings/mcp.json` matches the Power server and approval policy.
- [ ] The private dogfood pack is original, underscore-prefixed, and excluded from the public catalogue.
- [ ] Kiro evidence shows find → minimized counterexample → fix → validate → re-hunt and Power installation.
- [ ] Chat export, screenshots, and `docs/KIRO_USAGE.md` links are scrubbed and factual.
- [ ] Protocol, serialization, boundary, fake-runner, refusal, Power, static-pack, and real-sandbox tests pass.
- [ ] Rust lessons are recorded for every implementation task that changes Rust.
