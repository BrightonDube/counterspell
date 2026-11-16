# counterspell-mcp-power: technical decisions

Recorded during T01 (16 Nov 2026). Sources were read on that date.

## rmcp (official Rust MCP SDK)

- Crate: `rmcp`, latest release 3.10.0 (crates.io, published 12 Nov 2026). Licence Apache-2.0.
- Minimum Rust for 3.10.0 is 1.88 and its edition is 2024. The workspace declares `rust-version = "1.82"`. Decision: `crates/mcp` sets its own `rust-version = "1.88"` and does not inherit the workspace value. Other crates keep 1.82.
- Features to enable: `server`, `macros` (defaults), `transport-io` (server stdio transport). Do not enable `client`, `auth`, HTTP transports or `transport-child-process`, because the MCP must not open sockets or spawn children.
- Server API used: `ServiceExt::serve(stdio())`, then `service.waiting()`. The SDK example for stdio uses this pattern.
- Tool registration uses the SDK macros (`#[tool_router]`, `#[tool]`) with parameter types that derive `schemars::JsonSchema`. `schemars` is a required SDK dependency for tool input schemas and is added to `crates/mcp` only.

## Agent Plugins manifest (kiro-power/)

Checked against the normative schemas at agent-plugins.org (version 1.0.0).

- `plugin.json` is a closed schema. Allowed top-level keys: `$schema` (const `https://agent-plugins.org/schemas/1.0.0/plugin.schema.json`), `name` (lowercase, 1 to 64 characters, required), `version`, `description`, `author`, `homepage`, `repository`, `license`, `keywords`, `extensions`. The design's `mcpServers` or tool lists are not allowed at the top level.
- `mcp.json` must contain `$schema` (const `https://agent-plugins.org/schemas/1.0.0/mcp.schema.json`) and `mcpServers`. Each stdio server needs `type: "stdio"` and `command`, and may have `args`, `env` and `cwd`. `disabled` and `autoApprove` are not allowed on a server object.
- `cwd` must begin with `./`, `${PLUGIN_ROOT}` or `${PLUGIN_DATA}`. The design's `${workspaceFolder}` is not valid for a Power. Decision: the Power uses `cwd: "${PLUGIN_ROOT}"` and reaches the workspace manifest with `args` containing `--manifest-path ../Cargo.toml`. No absolute path is written.
- Auto-approval: the portable schema has no field for it, and Kiro's documentation checked on 16 Nov 2026 does not name a Power key. The Power's `mcp.json` therefore carries no approval list. The workspace `.kiro/settings/mcp.json` carries the exact approval list for the three pure tools. This is an open question for Brighton (see the blockers file).

## Engine and Grimoire seams (T02 to T04)

- The existing Grimoire trait `counterspell_grimoire::runner::PackRunner` (`run`, `bench`) is the typed runner contract. The judge is the only intended implementation. The MCP receives it by injection.
- The engine `Oracle` trait is reached through `counterspell_grimoire::runner::ProgramOracle`, which adapts a `PackRunner` and one incorrect program.
- Production construction of a `PackRunner` needs the judge's sandbox executor (judge-sandbox T20 to T24). It is not built, so the MCP has no production runner. Code-running tools return `sandbox_not_configured` until it exists. The MCP never falls back to a host interpreter or to a Docker CLI.

## Judge library boundary (T05)

- The design asks for a library target in `counterspell-judge` with a runner module shared with the worker. Its production implementation depends on judge-sandbox T20 to T24, which are not done. T05 is blocked; see the blockers file.

## Dogfood pack metadata (T15)

- The Grimoire `meta.toml` model uses `deny_unknown_fields`. A `dogfood = true` key would be rejected. Decision: the dogfood pack omits that key. Its private status comes from the underscore-prefixed directory `packs/_demo/`, which the public importer skips.

## Decisions made during T02 to T10 (16 Nov 2026)

- **Crate `sha2` 0.10 added to `counterspell-mcp`.** The hunt identity needs the SHA-256 of the program bytes (`code_sha256`), and the engine has no hash helper. `sha2` is already used by the judge at the same version, so no new licence appears.
- **Grimoire visibility change.** `ghosts::sandbox_ghosts`, `SandboxGhost` and its fields became `pub` (with `#[must_use]` on the function). The MCP needs a pack's authored known ghosts for `find_ghost_input` and `explain_verdict`. No behaviour changed.
- **The judge is reused for one function.** `counterspell-judge` is a dependency of the MCP for `sandbox::daemon::validate_socket_path` (main-daemon socket refusal) and `languages::LanguageCatalog::load` (pinned Python image). No judge runner is used yet.
- **The pinned image is read once at start-up.** `LanguageCatalog::load` currently fails in this repository because `languages/drivers/` does not exist (judge-sandbox and language-drivers blockers). The server therefore starts with no image digest, and `find_ghost_input` returns `sandbox_not_configured` until the catalogue loads. The tests pass a fixed fake digest.
- **Hunt target is the engine default.** `find_ghost_input` has no target argument, so the engine hunts the first wrong answer or crash. A program that only times out is reported as `NotFound`. The engine documents this (`hunt.rs`, `is_target`).
- **Code-running tools run on blocking threads.** The `PackRunner` trait is synchronous, so each code-running call runs in `spawn_blocking`. The runner lock is never held across an await.
