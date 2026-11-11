---
inclusion: always
---
# Counterspell technology decisions

Use the source documents below for implementation detail. These decisions are fixed.

## Decided - do not change without asking Brighton
- Rust/Axum single server binary plus a trusted Rust judge worker that owns tests, comparison, deadlines, and verdicts.
- Postgres is truth, including the job queue with `FOR UPDATE SKIP LOCKED` and leases, and persisted `run_events`.
- Valkey is for cache and rate limits only. It is not the queue or event bus.
- Docker is controlled through bollard on a separate rootless daemon used only for sandboxes. The server cannot access a Docker daemon.
- Canonical Cargo packages: `counterspell-{server,judge,engine,grimoire,mcp,common}`.
- Python and JavaScript launch first. Java and C++ are stretch goals.
- SvelteKit with the static adapter; Bits UI; Tailwind 4; Monaco.
- GSAP is limited to the landing page and Familiar; keep the IDE calm and use reduced-motion alternatives.
- MIT licence.
- OpenAI-compatible hints use `gpt-6-luna`, then Groq `openai/gpt-oss-20b`.

## Engineering rules
- Keep untrusted code inside the sandbox, including all code-running MCP tools.
- Validate inputs server-side, persist public run events in Postgres, and replay SSE by sequence number.
- Prefer small, testable changes and existing workspace conventions. Do not add a second architecture or crate layout.

Source of truth:
#[[file:docs/system-design.md]]
#[[file:docs/licence-rules.md]]
#[[file:docs/requirements.md]]
