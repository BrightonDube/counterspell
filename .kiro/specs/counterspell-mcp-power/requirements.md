# Requirements: counterspell-mcp-power

## Purpose

Provide the developer-facing Counterspell MCP server and installable Kiro Power described by `docs/system-design.md` §8 and `docs/kiro-playbook.md` A.5–A.6. Kiro should be able to validate original Grimoire packs, hunt and shrink ghost inputs, inspect verdict explanations, and fit supplied complexity timings through the same engine used by production.

The feature also provides a dogfooding proof: Kiro asks `find_ghost_input` to investigate a deliberately buggy implementation through the sandbox, fixes the bug, and re-verifies the pack with `validate_pack` and `find_ghost_input`. The proof must produce evidence that can be used in the Kiroween video and `docs/KIRO_USAGE.md`.

Sources: `docs/system-design.md` §§2, 5, 6, 7, 8 and 8b; `docs/requirements.md` MCP-1 to MCP-3 and the Sandbox criteria; `docs/kiro-playbook.md` A.5, A.6 and A.10; `.kiro/specs/ghost-engine/`; `.kiro/specs/grimoire-packs/`; `.kiro/steering/structure.md`, `security.md`, `sandbox.md`, and `rust-teaching.md`.

## Scope constraints

- The canonical package remains `crates/mcp/` with package name `counterspell-mcp`. No seventh Cargo package, alternate MCP crate, or parallel crate layout is allowed.
- The MCP server uses the official Rust SDK, `rmcp`, and stdio transport.
- Ghost generation, classification, comparison, shrinking, autopsy data, complexity fitting, and result wording come from `counterspell-engine` or the established Grimoire library. MCP contains adapters and protocol schemas, not duplicate algorithms.
- The trusted sandbox runner is owned by `counterspell-judge`. If the judge needs a reusable library target, it is added to the existing `counterspell-judge` package; a second judge crate is forbidden.
- `run_reference`, `find_ghost_input`, and `shrink_input` execute code only through the judge runner on the separate rootless Docker daemon reserved for sandboxes. They never spawn Python, Node, a shell, or a pack program on the developer's machine.
- `validate_pack` is static-only in this feature. It reads pack files and calls the Grimoire static validator; it does not offer a sandbox flag or execute pack code. Sandbox pack validation remains a judge-backed Grimoire integration and is not hidden behind an auto-approved MCP operation.
- `fit_complexity` accepts measured timing samples and performs a pure fit. It does not benchmark code. Its fitting implementation belongs to the engine and is shared with the Efficiency Familiar path.
- The Power is committed under `kiro-power/` in Agent Plugins format. It does not create a second MCP configuration language or a second skill system.
- The dogfood pack is underscore-prefixed and excluded from the public catalogue. It must be original and must not change the three ghost-engine fixture packs or the public starter-pack count.
- This spec describes implementation work only. The spec-generation run creates no implementation, Power, settings, pack, evidence, or documentation file outside this spec directory.

## Glossary

- **MCP server**: the `counterspell-mcp` stdio process started by Kiro.
- **Code-running tool**: an MCP tool that causes a reference or implementation to execute. In this feature these are `run_reference`, `find_ghost_input`, and `shrink_input`.
- **Trusted runner**: the judge-owned adapter that creates and destroys locked-down sandbox containers, enforces deadlines and output limits, and returns raw program outcomes.
- **Minimized counterexample**: the smallest failing input found within the bounded engine search and shrink budgets. It is never described as a proven global minimum or “smallest possible” input.
- **Static validation**: pack validation that reads text and structured data without interpreting or executing Python/JavaScript.
- **Read-only tool**: a tool that does not execute code, modify repository files, modify a database, or call an external service. Read-only status is distinct from whether a tool is safe to auto-approve.
- **Dogfood pack**: the private underscore-prefixed pack used to demonstrate Kiro using Counterspell MCP to find, fix, and re-verify a bug.

## Requirements

### MCP package and transport

1. **MCP-1** THE SYSTEM SHALL provide a binary package at `crates/mcp/` named `counterspell-mcp`.
2. **MCP-2** WHEN `counterspell-mcp` starts with a valid configuration, THE SYSTEM SHALL expose the registered tools over the official `rmcp` stdio transport and SHALL not open an HTTP, TCP, or public listener.
3. **MCP-3** THE SYSTEM SHALL use typed request and response schemas for every tool. Malformed JSON, missing required fields, unknown unsafe values, and invalid budgets SHALL produce a bounded structured tool error rather than a panic.
4. **MCP-4** THE SYSTEM SHALL preserve the MCP protocol boundary: stdout is reserved for protocol messages, while diagnostics and startup logs use stderr through the selected tracing configuration.
5. **MCP-5** IF the sandbox-only runner is unavailable, THE SYSTEM SHALL keep static tools available where possible but SHALL return an explicit infrastructure/configuration error for every code-running tool. It SHALL never fall back to a host interpreter, a shell, a local subprocess, or the main Docker daemon.

### Tool catalogue

6. **TOOL-1 `validate_pack`** WHEN called with a workspace-relative pack path, THE SYSTEM SHALL run the Grimoire static validator and return its structured diagnostic report, including stable diagnostic codes, severity, file, pointer/line when available, checked stages, skipped stages, and summary. It SHALL not execute any pack code.
7. **TOOL-2 `find_ghost_input`** WHEN called with a pack path and an implementation selected from that pack, THE SYSTEM SHALL run the engine hunt through a judge-provided sandbox Oracle, compare the implementation with the trusted reference using the pack comparator, and return a `Found`, `NotFound`, or infrastructure/invalid-request outcome with provenance and budget information.
8. **TOOL-3 `shrink_input`** WHEN called with a valid starting input, implementation, and target failure predicate, THE SYSTEM SHALL adapt the engine's existing pure shrinker to a judge sandbox probe. It SHALL retain only inputs that reproduce the requested failure predicate and SHALL return the recorded shrink trace and final minimized counterexample when one is verified.
9. **TOOL-4 `run_reference`** WHEN called with a pack path and schema-valid public input, THE SYSTEM SHALL execute the pack's reference through the judge sandbox and return the raw trusted outcome, comparator metadata, timing metadata allowed by the public result schema, and bounded diagnostics. It SHALL not execute the reference on the host.
10. **TOOL-5 `explain_verdict`** WHEN called with an observed result or ghost report, THE SYSTEM SHALL return an engine-derived explanation, structural difference, failure kind, and matching authored known-ghost explanation when one exists. It SHALL not invent a fix, call an LLM, execute code, or claim a global minimum.
11. **TOOL-6 `fit_complexity`** WHEN called with bounded measured timing samples and an optional declared reference class, THE SYSTEM SHALL call the engine's pure fit implementation and return the selected model, slope, ratios, residuals, confidence, and comparison to the declared class. It SHALL reject insufficient or malformed samples and SHALL not benchmark programs.
12. **TOOL-7** THE description registered for each of `run_reference`, `find_ghost_input`, and `shrink_input` SHALL state that code is executed only by the trusted judge in a locked-down sandbox on the sandbox-only rootless Docker daemon, that expected outputs/comparison/deadlines belong to the trusted judge, and that code is never run directly on the developer machine.
13. **TOOL-8** THE descriptions for `validate_pack`, `explain_verdict`, and `fit_complexity` SHALL state their non-execution behavior so Kiro can distinguish pure/read-only tools from code-running tools.
14. **TOOL-9** THE SYSTEM SHALL bound every tool's request size, source/pack path length, serialized response, output snippet, candidate budget, shrink budget, and wall-time request before handing work to the engine or judge. The server-side production limits remain the upper bound; MCP arguments cannot raise them.
15. **TOOL-10** THE SYSTEM SHALL never return hidden generated tests, hidden expected outputs, secrets, Docker socket paths, host filesystem details, or unbounded stderr in an MCP result.

### Engine, Grimoire, and judge reuse

16. **REUSE-1** THE SYSTEM SHALL call the public `counterspell-engine` hunt, classifier, comparator, shrinker, autopsy, schema, provenance, and result APIs rather than copy their algorithms into `crates/mcp`.
17. **REUSE-2** THE SYSTEM SHALL call the `counterspell-grimoire` library for pack loading and static validation rather than shell out to the Grimoire CLI or parse a second pack format.
18. **REUSE-3** THE SYSTEM SHALL expose the existing judge sandbox capability through the existing `counterspell-judge` package. The MCP adapter SHALL use the same language catalog, image digests, driver templates, limits, comparator, deadline enforcement, output caps, cleanup, and infrastructure-error mapping as production.
19. **REUSE-4** THE trusted runner SHALL own expected outputs, comparison, per-test deadlines, and final failure classification. Program output that resembles a verdict SHALL remain only a candidate answer.
20. **REUSE-5** Infrastructure failures SHALL remain distinct from user failures. A missing image, Docker/API error, lost sandbox, runner timeout, or configuration error SHALL become an MCP infrastructure error and SHALL never become `wrong`, `crash`, `timeout`, `NotFound`, or a successful validation result.
21. **REUSE-6** `fit_complexity` SHALL use the same pure fit model and confidence rules as the Efficiency Familiar implementation. It SHALL not create a second timing classifier.

### Sandbox and host-execution security

22. **SEC-1** THE SYSTEM SHALL run code-running tools only on the separate rootless Docker daemon used exclusively for sandboxes, never on the main Compose daemon and never on the developer host.
23. **SEC-2** The judge runner SHALL apply the production sandbox controls: no network, read-only root, bounded writable scratch space, non-root user, all capabilities dropped, no-new-privileges, default seccomp, CPU/memory/PID limits, worker-enforced wall/per-test deadlines, bounded stdout/stderr, and cleanup/reaping.
24. **SEC-3** THE MCP server SHALL accept only workspace-relative paths that resolve inside the configured workspace and pack root. It SHALL reject symlinks, path traversal, absolute paths, unexpected file types, and paths outside the selected pack.
25. **SEC-4** THE MCP and judge code SHALL not construct shell commands from tool arguments. The MCP implementation SHALL not use `std::process::Command` or equivalent to run Python, Node, a driver, or a pack program.
26. **SEC-5** THE SYSTEM SHALL include tests or static checks proving that the MCP code-running path cannot silently use a host interpreter or main Docker socket, and that no host fallback is attempted when sandbox configuration is absent.
27. **SEC-6** Pack data, tool arguments, program output, exception text, and diagnostics SHALL be treated as untrusted. Responses SHALL be structured or plain text, bounded, and safe for Kiro to display.

### Power packaging

28. **POWER-1** THE SYSTEM SHALL provide `kiro-power/plugin.json` in the Agent Plugins format with a Counterspell name/description and keywords including `ghost input`, `grimoire`, `problem pack`, and `shrink`.
29. **POWER-2** THE SYSTEM SHALL provide `kiro-power/mcp.json` registering the `counterspell-mcp` stdio server with a repository-relative command that works from a clean checkout and does not contain secrets.
30. **POWER-3** THE SYSTEM SHALL provide `kiro-power/skills/author-problem/SKILL.md` explaining how to create an original Grimoire pack, declare the schema/generator/reference/hints, run static validation, and use MCP instead of running pack code on the host.
31. **POWER-4** THE SYSTEM SHALL provide `kiro-power/skills/hunt-ghost/SKILL.md` explaining how to reproduce a failure, call `find_ghost_input`, inspect a minimized counterexample and autopsy, fix the implementation, and re-verify. The skill SHALL require the honest “minimized counterexample” wording.
32. **POWER-5** THE Power documentation SHALL distinguish read-only tools from code-running tools and SHALL never instruct Kiro to auto-approve or directly execute code-running tools.
33. **POWER-6** THE Power SHALL be importable from GitHub through Kiro's custom Power flow after implementation, with no machine-specific absolute paths or credentials.

### Workspace MCP registration

34. **CONFIG-1** THE SYSTEM SHALL register `counterspell-mcp` in `.kiro/settings/mcp.json` using the same stdio command and tool names documented by the Power.
35. **CONFIG-2** THE workspace configuration SHALL auto-approve only `validate_pack`, `explain_verdict`, and `fit_complexity`, because those tools are static/pure and non-executing in this feature.
36. **CONFIG-3** THE workspace configuration SHALL not auto-approve `run_reference`, `find_ghost_input`, or `shrink_input`. They require an explicit approval because they execute code, even though the judge sandbox is the required execution boundary.
37. **CONFIG-4** THE SYSTEM SHALL commit no secrets, tokens, credentials, private Docker socket credentials, or `.env` files in MCP or Power configuration.

### Dogfooding demonstration and evidence

38. **DEMO-1** THE SYSTEM SHALL provide a private demo pack at an underscore-prefixed path such as `packs/_demo/mcp-dogfood/`, with original statement/metadata/schema, a correct reference, one deliberately buggy implementation, known-ghost information where supported, and `dogfood = true`.
39. **DEMO-2** WHEN Kiro calls `find_ghost_input` for the buggy implementation, THE SYSTEM SHALL return a verified minimized counterexample from the sandbox, including failure kind, expected/actual data allowed for the demo, provenance, and actual recorded shrink steps.
40. **DEMO-3** AFTER the bug is fixed, THE scripted session SHALL call `validate_pack` and `find_ghost_input` again. Static validation SHALL pass, and the second ghost hunt SHALL report no counterexample within its configured budget or an equivalent successful re-verification result defined by the engine contract.
41. **DEMO-4** THE dogfood session SHALL show the sequence “find → inspect minimized counterexample → fix → validate → re-hunt” and SHALL not claim that the first counterexample is a proven global minimum.
42. **DEMO-5** THE implementation work SHALL save the scrubbed Kiro chat export under `docs/kiro-log/chats/`, screenshots under `docs/media/kiro/`, and a factual MCP/Power/dogfood entry in `docs/KIRO_USAGE.md`. The artifacts SHALL not contain secrets or hidden production test data.
43. **DEMO-6** THE evidence checklist SHALL include the MCP `find_ghost_input` call, the returned shrink trace, the code fix, the successful validation, the no-counterexample re-hunt, and the Power installation/import step.

### Tests and completion

44. **TEST-1** THE SYSTEM SHALL include protocol/schema tests for all six tools, including malformed input, unknown fields, missing fields, oversized values, invalid paths, invalid schemas, invalid budgets, and bounded output.
45. **TEST-2** THE SYSTEM SHALL include pure adapter tests proving MCP maps engine `Found`, `NotFound`, and `Aborted` results without changing their meaning or wording.
46. **TEST-3** THE SYSTEM SHALL include a fake judge-runner test double for deterministic MCP tests and a real sandbox smoke test for the code-running path. The fake must not weaken the production security boundary.
47. **TEST-4** THE SYSTEM SHALL include refusal tests proving no host interpreter is invoked, no shell is constructed from tool input, and missing sandbox configuration does not cause host fallback.
48. **TEST-5** THE SYSTEM SHALL validate the Power JSON, required keywords, skill paths, MCP command, workspace MCP configuration, auto-approval allow-list, and clean-checkout import assumptions.
49. **TEST-6** THE dogfood demonstration SHALL be reproducible from documented commands and SHALL pass the same validation and sandbox constraints as production.
50. **TEST-7** Rust implementation tasks SHALL follow the repository Rust teaching rules: no production `unwrap()`/`expect()`, useful contextual errors, simple APIs, and a 3–5 line Rust lesson recorded after each Rust change.

## Out of scope

- HTTP MCP transport, a public MCP endpoint, or a second server binary.
- A new Cargo package for the sandbox or MCP runner.
- Any direct host execution of Python, JavaScript, drivers, reference code, or incorrect implementations.
- Automatic account linking, OAuth, Postgres queue operations, Valkey queue/event use, server API routes, SSE, UI work, AI hints, and Familiar UI.
- Sandbox validation hidden inside auto-approved `validate_pack`; static validation is the only `validate_pack` mode in this feature.
- A second complexity model, a live typing estimator, or a tool that benchmarks code directly.
- Publishing a public dogfood pack in the catalogue.
- Java/C++ implementation beyond whatever the shared production language catalog already supports.
