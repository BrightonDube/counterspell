---
inclusion: always
---
# Security rules

Treat submissions, pack data, tool input, and program output as untrusted.

- Never read or write `.env`, key files, credentials, or secrets. Use environment configuration only in the running server.
- Add an ownership check to every user-scoped query and every run/event endpoint.
- OAuth must use `state` and PKCE. Never automatically link accounts by matching email; linking is user-initiated and verified.
- Enforce rate limits and queued-job caps. Enforce VAL-1 through VAL-5 server-side, not only in the browser.
- Follow `docs/system-design.md` §7 for the sandbox. Untrusted code never runs outside it, including code invoked by `counterspell-mcp`.
- Render stdout, stderr, and test output as plain text. Do not use keyword blocklists as the execution boundary.

Source:
#[[file:docs/system-design.md]]
#[[file:docs/requirements.md]]
