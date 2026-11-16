# counterspell-mcp-power: blockers and open items (16 Nov 2026)

## Open tasks and why

- **T05 (judge runner library, shared with the worker).** The judge's production sandbox runner (judge-sandbox T20 to T24) is not built, so there is no production `PackRunner`. T05 needs it. Unblock: judge-sandbox T20 to T24 done, with the `sbx_` tests passing on the rootless daemon.
- **T10 (code-running tools).** The fake-runner part is done and tested. The real rootless smoke test in its verify line needs T05. Unblock: the same as T05.
- **T11 (Power manifest).** The files and the schema-rule tests are done. The verify line also needs Kiro to import the Power. Brighton must run that import test.
- **T11 open question: auto-approval in a Power.** The Agent Plugins `mcp.schema.json` forbids `autoApprove` on a server, and Kiro's docs (checked 16 Nov 2026) name no Power key for it. The Power therefore ships without an approval list. The workspace file carries the pure-tool list. Brighton should say where Kiro reads an approval list for a Power.
- **T14 (`.kiro/settings/mcp.json`).** The write was refused by a `kiro-scope` permission rule (`deny fs_write`). I did not work around it. Brighton should either allow that path or create the file with the content in the T14 task (the file must hold only the three pure tools in `autoApprove`).
- **T15 (dogfood pack).** A PreToolUse hook (`LAY-05`, `PIP-31`) blocks private files under the public `packs/` folder. Brighton's 14 Nov decision also moves reference solutions, incorrect programs and known ghosts to the private repo. The design's `packs/_demo/mcp-dogfood/` therefore conflicts with that decision. I deleted the `reference/solution.py` and `incorrect/short-last-window.py` I had written. Still untracked and not committed: `packs/_demo/mcp-dogfood/problem.md`, `meta.toml` and `hints.toml`, which are public parts. Brighton should decide whether the hidden parts go to `BrightonDube/counterspell-packs-private` (the `_demo` directory then holds only public parts), and whether the MCP takes a private pack root.
- **T16 (scripted Kiro find, fix, re-verify session).** Needs a live Kiro chat with the Power or workspace MCP, run by Brighton. Blocked by T14, T15 and T10.
- **T17 (scrubbed evidence).** Needs T16 and the Kiro screenshots.
- **T18 (boundary validation).** Depends on T14 and T17, which are open. The static tests that need no open dependency can be written later.
- **T19 (full sandbox verification).** Needs a real rootless sandbox daemon with the judge runner and pinned images. Blocked by T05 and T10.

## Other notes

- **Power `cwd` and workspace root.** The Power uses `cwd: ${PLUGIN_ROOT}` with `--manifest-path ../Cargo.toml`. The server finds the repository root by walking up from its working directory (`config::workspace_root`), so it needs no environment variable. Kiro's handling of `cwd` in the workspace file is unverified.
- **Pinned image.** `languages/languages.toml` cannot load yet, because `languages/drivers/` does not exist (language-drivers T03/T04). The server starts with no image digest and `find_ghost_input` returns `sandbox_not_configured` until it loads.
- **Rust version.** `crates/mcp` sets `rust-version = 1.88` because `rmcp` 3.10.0 needs it. The workspace stays at 1.82.
- **Test-first.** The first version of `config.rs` and `runner.rs` was written with its tests in the same step, so I did not observe a red run before green. Later modules were run red-first where a test failed (the path-dot case, the digest ordering, the timeout expectation).
- **Timeout expectation corrected.** A test I wrote expected a timeout ghost. The engine documents that timeouts count only when a target asks for them, and `find_ghost_input` has no target, so the test now expects `NotFound`. This is my own new test, not an existing requirement test, so no `test-change` marker was used.
- **Review.** I did not start a sub-agent for `requesting-code-review`. Self-review was done against the checklist, with clippy `-D warnings` clean and fmt clean. Minor findings: `config.rs` still has a `#[must_use]` candidate in tests (none outstanding in clippy); `tools.rs` is long (about 900 lines with tests) and could be split into pure and code-running modules.
- **Sandbox endpoint.** `SANDBOX_DOCKER_SOCKET` is the variable name, matching the judge. The MCP validates it with the judge's guard, which refuses the main daemon socket.
