# Requirements: repo-skeleton

## Purpose

Create the first runnable Counterspell repository skeleton from the fixed architecture in `docs/system-design.md`. The skeleton establishes the canonical Cargo workspace, a minimal Axum server, a SvelteKit static frontend, local Compose infrastructure, migration wiring, deployment boundary documentation, licensing files, and CI. It must not implement later product features.

## Scope constraints

- The workspace SHALL contain exactly the six canonical Cargo packages listed below and SHALL NOT introduce alternate crate names or parallel package layouts.
- The server SHALL serve the built SvelteKit application from the Rust binary.
- Valkey SHALL be used only for cache and rate-limit storage. It SHALL NOT be the job queue or event bus.
- The server SHALL not access a Docker daemon. The future judge SHALL be isolated from the main Docker socket.
- Untrusted code execution, authentication, judging, ghost hunting, run events, hints, dashboard features, and visual feature work are outside this skeleton and belong to later specs.

## EARS acceptance criteria

### Workspace and canonical layout

1. THE SYSTEM SHALL have a root `Cargo.toml` workspace with exactly these members and package names:
   - `crates/server/` → `counterspell-server`
   - `crates/judge/` → `counterspell-judge`
   - `crates/engine/` → `counterspell-engine`
   - `crates/grimoire/` → `counterspell-grimoire`
   - `crates/mcp/` → `counterspell-mcp`
   - `crates/common/` → `counterspell-common`
2. THE SYSTEM SHALL preserve the package responsibilities from the architecture: server/API/static files, trusted judge worker, pure ghost engine, Grimoire pack validator, MCP stdio server, and shared types respectively.
3. THE SYSTEM SHALL establish the canonical top-level paths needed by the architecture: `crates/`, `web/`, `languages/`, `packs/`, `migrations/`, `deploy/`, `.kiro/`, `docker-compose.yml`, `Caddyfile`, `README.md`, `LICENSE`, and `THIRD-PARTY-NOTICES.md`.
4. THE SYSTEM SHALL NOT add a seventh Cargo package, a second server package, or a separate frontend repository.

### Axum server

5. WHEN `counterspell-server` starts with a valid configuration, THE SYSTEM SHALL start an Axum/Tokio HTTP server.
6. WHEN a client requests `GET /api/health`, THE SYSTEM SHALL return a successful JSON health response without requiring authentication or database state.
7. WHEN a client requests a built frontend asset, THE SYSTEM SHALL serve it from the SvelteKit adapter-static output through the Rust process using `tower-http` static-file serving or an equivalent Axum-compatible implementation.
8. WHEN a browser requests a non-API frontend route that has no matching static asset, THE SYSTEM SHALL return the SvelteKit entry point so client-side navigation can boot.
9. THE SYSTEM SHALL give `/api/health` and future `/api/*` routes precedence over the static-file and SPA fallback routes.
10. IF the configured static directory is unavailable or the server cannot bind its configured address, THEN the process SHALL fail clearly rather than claim readiness.

### SvelteKit frontend

11. THE SYSTEM SHALL contain a SvelteKit 5 application in `web/` using the static adapter and producing a deployable `build/` directory.
12. THE SYSTEM SHALL configure Tailwind 4 semantic tokens from `docs/palette.md`, including the Steel Night dark-mode tokens for background, surfaces, text, moss, candle, info, success, danger, and focus.
13. THE SYSTEM SHALL use the existing frontend direction of SvelteKit, Bits UI-compatible primitives, and Tailwind 4 without creating a second styling or component architecture.
14. WHEN the generated frontend is opened through the server, THE SYSTEM SHALL display a real minimal Svelte page and SHALL NOT present fabricated judging results.
15. THE SYSTEM SHALL keep semantic colour names in frontend components and SHALL not use raw palette hex values as component-level styling decisions.

### Compose and service boundaries

16. THE SYSTEM SHALL define a `docker-compose.yml` containing the first-milestone services `caddy`, `server`, `judge`, `postgres`, `valkey`, and one-shot `migrate`.
17. THE `caddy` service SHALL publish ports 80 and 443, persist `caddy_data` and `caddy_config`, and reverse proxy application traffic to the server.
18. THE `server` service SHALL be built from a multi-stage image that builds the SvelteKit static output and the Rust server, then includes the static output in the runtime image.
19. THE `postgres` service SHALL use the PostgreSQL 17 image, persist `pgdata`, and be reachable only through internal Compose networking.
20. THE `valkey` service SHALL use the Valkey 8 image and be reachable only through internal Compose networking. Its documentation and configuration SHALL identify it as cache/rate-limit storage only.
21. THE `judge` service SHALL have no published ports and SHALL be attached only to the required internal network in the skeleton.
22. THE `judge` service SHALL not receive the main Docker daemon socket. The `server`, `postgres`, `valkey`, and `caddy` services SHALL not receive the future sandbox daemon socket.
23. THE `migrate` service SHALL be one-shot, wait for Postgres readiness, run `sqlx migrate run` against the `migrations/` directory, and exit with a failure status when migrations fail.
24. IF a required dependency is unavailable, THEN Compose startup or the affected service SHALL expose a clear health/startup failure rather than silently reporting a working stack.

### Migrations

25. THE SYSTEM SHALL contain a usable `migrations/` directory for SQLx migrations.
26. THE initial migration SHALL be valid and minimal, reserving domain tables for later feature specs.
27. THE migration path SHALL be usable both by the Compose `migrate` service and by local development commands.

### Sandbox deployment boundary

28. THE SYSTEM SHALL contain a `deploy/` note describing a separate rootless Docker daemon used only for sandboxes.
29. THE note SHALL specify an unprivileged `sandbox` user, a systemd user service, a separate socket, pre-pulled sandbox images, and cgroup v2 delegation verification.
30. THE note SHALL state that only the trusted judge may later mount the sandbox socket, that the server cannot access any Docker daemon, and that the main Compose daemon is not the sandbox boundary.
31. THE skeleton SHALL document `JUDGE_CONCURRENCY=2` as the default until VPS benchmarking permits a change.

### CI and licensing

32. WHEN a GitHub Actions workflow runs on push, THE SYSTEM SHALL run `cargo fmt --check`.
33. WHEN the workflow checks Rust code, THE SYSTEM SHALL run Clippy with warnings denied and `cargo test` for the workspace.
34. WHEN the workflow checks the frontend, THE SYSTEM SHALL run `npm run check` and `npm run build` from `web/`.
35. WHEN the workflow checks dependency licensing, THE SYSTEM SHALL run `cargo deny check licenses`.
36. THE SYSTEM SHALL provide a detectable MIT `LICENSE` at the repository root.
37. THE SYSTEM SHALL provide `THIRD-PARTY-NOTICES.md` for dependency and asset notices and SHALL not silently claim notices are complete for dependencies not yet installed.
38. THE SYSTEM SHALL keep CI commands reproducible with pinned or lockfile-resolved dependencies where the selected tools support it.

### End-to-end completion

39. WHEN a developer runs `docker compose up`, THE SYSTEM SHALL start the local stack sufficiently for a browser or `curl` request to `http://localhost/` to receive the Svelte page through Caddy and `counterspell-server`.
40. WHEN a developer requests `http://localhost/api/health`, THE SYSTEM SHALL receive a successful health response through the same path.
41. THE served page SHALL be proven to come from the Rust server's static-file route and not from a separate frontend development server.
42. THE SYSTEM SHALL pass the same formatting, lint, test, frontend, build, and license checks required by CI.
43. The repository skeleton is complete only when the local Compose smoke test succeeds and CI is green. This criterion does not claim that the later judge, sandbox, authentication, ghost engine, or full product API is implemented.

## Out of scope

- OAuth, email authentication, sessions, account linking, or demo accounts.
- Job claiming, leases, run events, SSE, test comparison, language drivers, or sandbox execution.
- Ghost generation/shrinking, Grimoire pack content, MCP tools, hints, dashboard, Familiar, landing animation, or IDE features.
- Java/C++ support, payments, FSRS, timed trials, and break-room features.
