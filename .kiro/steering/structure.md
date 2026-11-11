---
inclusion: always
---
# Counterspell repository structure

Use one canonical layout. Do not invent parallel crate names or move responsibilities without asking Brighton.

- `crates/server/` → `counterspell-server`: Axum API, auth, SSE, and static web files.
- `crates/judge/` → `counterspell-judge`: trusted worker, queue claimer, sandbox runner, comparator, and verdicts.
- `crates/engine/` → `counterspell-engine`: pure ghost generation, comparison, and shrinking.
- `crates/grimoire/` → `counterspell-grimoire`: problem-pack format and validator.
- `crates/mcp/` → `counterspell-mcp`: rmcp stdio server; code-running tools use the judge sandbox.
- `crates/common/` → `counterspell-common`: shared IDs, requests, and run-event types.
- `web/`: SvelteKit 5 static app, served by the server; use `web/**` guidance.
- `packs/`: original Grimoire problem packs; use pack authoring guidance.
- `languages/`: `languages.toml` and language drivers; use sandbox guidance.
- `migrations/`: SQLx migrations for Postgres.
- `deploy/`: Compose, rootless sandbox-daemon setup, hardening, and backups.
- `.kiro/`: committed steering, specs, hooks, agents, settings, and MCP configuration.

The root workspace and package names are authoritative. Keep deployment services and sandbox boundaries aligned with the system design.

Source of truth:
#[[file:docs/system-design.md]]
#[[file:docs/kiro-playbook.md]]
