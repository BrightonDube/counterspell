---
inclusion: auto
name: API conventions
description: Use for Axum REST endpoints, authentication, sessions, run events, or SSE client/server work.
---
# API conventions

- Return JSON for API successes and errors. Errors use a stable machine-readable `code` and a safe human-readable `message`; never include secrets, hidden tests, or raw sensitive data.
- Use cookie-backed sessions with HttpOnly, Secure, SameSite=Lax settings and ownership checks before returning user or run data.
- Use the documented SSE endpoint with `Last-Event-ID`; event IDs are persisted per-run sequence numbers and clients deduplicate by sequence.
- Supported event names are `compiling`, `test`, `ghost_hunting`, `shrink_step`, `ghost`, `bench_point`, `complexity`, `familiar`, `restarted`, `done`, and `error`.
- Keep hidden inputs and expected outputs out of public events. Render program output as plain text.

Source:
#[[file:docs/system-design.md]]
