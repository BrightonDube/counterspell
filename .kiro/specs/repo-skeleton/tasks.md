# Tasks: repo-skeleton

All tasks are implementation tasks for the design above. No task in this document is being executed by this spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its listed dependencies are complete. `[S]` means it is sequential or has a required dependency.

## Wave 0: foundations

- [x] **T01 [P] (20 min) Add repository licensing files**
  - Files: `LICENSE`, `THIRD-PARTY-NOTICES.md`.
  - Create the MIT license and a notice template that distinguishes current dependencies from future notices.
  - Depends on: none.
  - Verify: files exist at the repository root and contain no placeholder license other than the intended notice template.
  - Preserve: MIT licensing decision; do not add unapproved dependency claims.

- [x] **T02 [P] (20 min) Add repository orientation files**
  - Files: `README.md`, canonical empty directories `languages/`, `packs/`, `migrations/`, and `deploy/` as needed.
  - Document the project purpose, local commands, six-package layout, and the fact that judging/sandbox work is later.
  - Depends on: none.
  - Verify: README names the canonical packages and does not describe Valkey as a queue.
  - Preserve: canonical repository structure.

- [x] **T03 [S] (25 min) Create the exact Cargo workspace manifest**
  - Files: `Cargo.toml`, `Cargo.lock` after dependency resolution.
  - Declare exactly `crates/server`, `judge`, `engine`, `grimoire`, `mcp`, and `common`; set shared Rust edition/version/lint conventions without adding packages.
  - Depends on: none.
  - Verify: `cargo metadata --no-deps` lists exactly six workspace packages with the required names.
  - Preserve: no alternate crate names or parallel workspace.

## Wave 1: independent package and frontend scaffolds

- [x] **T04 [P] (25 min) Scaffold the common and engine packages**
  - Files: `crates/common/Cargo.toml`, `crates/common/src/lib.rs`, `crates/engine/Cargo.toml`, `crates/engine/src/lib.rs`.
  - Create minimal compiling libraries with documented ownership boundaries and no process/Docker dependencies.
  - Depends on: T03.
  - Verify: `cargo check -p counterspell-common -p counterspell-engine`.
  - Preserve: common is shared types only; engine is pure.

- [x] **T05 [P] (25 min) Scaffold judge, Grimoire, and MCP packages**
  - Files: `crates/judge/**`, `crates/grimoire/**`, `crates/mcp/**`.
  - Create the exact package names, minimal entry points, and a single Grimoire package containing its future `validate` binary rather than a second package.
  - Depends on: T03.
  - Verify: `cargo check -p counterspell-judge -p counterspell-grimoire -p counterspell-mcp`.
  - Preserve: judge is the future sandbox boundary; MCP never runs code directly on the host.

- [x] **T06 [P] (30 min) Create the SvelteKit static application**
  - Files: `web/package.json`, lockfile, SvelteKit config, `web/src/routes/+page.svelte`, `web/src/app.css`, and required config files.
  - Configure SvelteKit 5 with adapter-static, Tailwind 4, and the selected Bits UI-compatible foundation. Add a real minimal page with a stable smoke-test marker.
  - Depends on: none.
  - Verify: `cd web && npm ci && npm run check && npm run build`.
  - Preserve: `web/` is the only frontend application; no dev server is part of production Compose.

- [x] **T07 [P] (25 min) Add Steel Night semantic Tailwind tokens**
  - Files: `web/src/app.css` and frontend configuration only.
  - Translate the semantic tokens from `docs/palette.md` into Tailwind 4 CSS-first theme variables and a dark launch theme. Use semantic classes in the page.
  - Depends on: T06.
  - Verify: `cd web && npm run check && npm run build`; inspect that raw component hex values are absent.
  - Preserve: moss/candle general accents, green success-only, red failure-only, no second theme architecture.

## Wave 2: server and migration

- [x] **T08 [S] (30 min) Scaffold the Axum server package**
  - Files: `crates/server/Cargo.toml`, `crates/server/src/main.rs`, and a small module if needed.
  - Add Tokio/Axum startup, environment configuration with local defaults, and `GET /api/health` JSON response. Avoid `unwrap()`/`expect()` in production code.
  - Depends on: T03.
  - Verify: `cargo check -p counterspell-server`; start it locally and curl `/api/health`.
  - Preserve: server has no Docker client/socket and health does not pretend to be full readiness.

- [x] **T09 [S] (30 min) Add static serving and SPA fallback**
  - Files: `crates/server/src/main.rs` or server modules and static-serving dependencies.
  - Serve `STATIC_DIR` with `ServeDir`, preserve API route precedence, serve `index.html` for non-API frontend routes, and return clear startup errors for missing/bind failures.
  - Depends on: T08 and T06.
  - Verify: build `web/`, run the server, curl `/`, a static asset, `/some-client-route`, and `/api/health`.
  - Preserve: frontend is served from the Rust process, never a dev server; unknown API routes do not become SPA HTML.

- [x] **T10 [P] (25 min) Add the minimal SQLx migration path**
  - Files: `migrations/<timestamp>_bootstrap.sql`.
  - Add one valid, idempotently runnable bootstrap migration suitable for proving migration execution, without domain tables.
  - Depends on: T03.
  - Verify: run the migration with SQLx against a local Postgres instance or the eventual Compose database.
  - Preserve: Postgres remains the future truth, queue, and event store; no queue/event logic is placed in Valkey.

## Wave 3: image and local infrastructure

- [x] **T11 [P] (25 min) Write the rootless sandbox deployment note**
  - Files: `deploy/rootless-sandbox-daemon.md`.
  - Document the unprivileged `sandbox` user, systemd user service, separate socket, pre-pulled images, cgroup v2 delegation check, judge-only access, no server socket, and `JUDGE_CONCURRENCY=2`.
  - Depends on: none.
  - Verify: note explicitly distinguishes the main Compose daemon from the sandbox daemon and contains no host-socket mount instruction for the server.
  - Preserve: separate rootless sandbox daemon is a fixed security boundary; do not implement code execution.

- [x] **T12 [S] (30 min) Create the multi-stage Dockerfile**
  - Files: `Dockerfile`.
  - Add Node web-build, Rust release-build, server runtime, judge runtime, and pinned SQLx migration tooling targets. Copy `web/build` into the server runtime and set `STATIC_DIR` appropriately.
  - Depends on: T04, T05, T06, T08, T09, T10.
  - Verify: build the server, judge, and migration targets; inspect that the server runtime has the static build and no Docker socket.
  - Preserve: server does not access Docker; only later judge deployment can receive the sandbox socket.

- [x] **T13 [S] (30 min) Add Caddy and Compose topology**
  - Files: `Caddyfile`, `docker-compose.yml`.
  - Define `caddy`, `server`, `judge`, `postgres:17`, `valkey/valkey:8`, and one-shot `migrate`, with internal networks, health checks, named volumes, service dependencies, and only Caddy ports 80/443 published.
  - Depends on: T10 and T12.
  - Verify: `docker compose config`; inspect that judge has no published ports and no main Docker socket, and Valkey has no queue/event service configuration.
  - Preserve: Caddy proxies to Axum; Postgres is truth; Valkey is cache/rate limits only.

- [x] **T14 [S] (25 min) Wire Compose migration execution**
  - Files: `docker-compose.yml`, `Dockerfile` if target details need correction.
  - Make `migrate` wait for Postgres health, run `sqlx migrate run --source /app/migrations`, and fail visibly when migration execution fails.
  - Depends on: T10, T12, T13.
  - Verify: bring up Postgres and migration service, inspect a successful migration, then verify a deliberately invalid migration fails non-zero without claiming success.
  - Preserve: no committed secrets; migration execution is one-shot, not a second API service.

## Wave 4: CI and integration

- [x] **T15 [P] (30 min) Add GitHub Actions CI**
  - Files: `.github/workflows/ci.yml`.
  - Run Rust fmt check, Clippy with warnings denied, workspace tests, web `npm ci`, `npm run check`, `npm run build`, and `cargo deny check licenses` on push. Use lockfile-resolved dependencies.
  - Depends on: T03, T04, T05, T06.
  - Verify: run equivalent commands locally and validate workflow YAML.
  - Preserve: CI checks exactly the selected stack; do not add unrelated deploy automation.

- [x] **T16 [S] (25 min) Add local server/frontend image integration**
  - Files: `Dockerfile`, `docker-compose.yml`, server config only if needed.
  - Ensure the server image contains the Svelte static build and starts `counterspell-server` with the correct static directory; ensure Caddy reaches it.
  - Depends on: T09, T12, T13.
  - Verify: `docker compose build server` and inspect/curl the built service.
  - Preserve: the page must be served by Axum, not by an npm process.

- [ ] **T17 [S] (30 min) Run the Compose smoke test**
  - Files: only implementation fixes required by the smoke test; do not broaden scope.
  - Run `docker compose up`, request `http://localhost/`, request `http://localhost/api/health`, and confirm the response includes the Svelte marker and health JSON.
  - Depends on: T14 and T16.
  - Verify: Caddy logs show proxying to `server:8080`; no frontend dev server is exposed; judge has no public port or main Docker socket.
  - Preserve: local success must not weaken the sandbox boundary or expose database/Valkey ports.

- [ ] **T18 [S] (30 min) Run the complete CI-equivalent verification**
  - Files: no planned changes; fixes only if a check fails.
  - Run `cargo fmt --all -- --check`, Clippy with `-D warnings`, `cargo test --workspace`, `cd web && npm run check && npm run build`, `cargo deny check licenses`, and `docker compose config`.
  - Depends on: T15 and T17.
  - Verify: every command passes and the final repository state matches the exact six-package layout.
  - Preserve: done means Compose serves the page from Rust and CI is green; do not start later feature work.

## Dependency summary and parallel waves

- **Wave 0:** T01 and T02 can run in parallel. T03 is the workspace foundation.
- **Wave 1:** after T03, T04 and T05 can run in parallel. T06 can run in parallel with T03–T05 because the frontend is independent. T07 follows T06.
- **Wave 2:** T08 follows T03. T09 follows T08 and T06. T10 can run in parallel with T08/T09.
- **Wave 3:** T11 can run in parallel with all implementation work. T12 follows the package/server/frontend build inputs. T13 follows T10 and T12. T14 follows T13.
- **Wave 4:** T15 can begin once the workspace and frontend manifests exist. T16 follows T09/T12/T13. T17 and T18 are sequential final gates.

## Completion checklist

- [ ] Exactly six Cargo packages exist with the required names.
- [ ] `GET /api/health` works through Axum.
- [ ] `docker compose up` serves the Svelte page at `http://localhost/` from the Rust binary.
- [ ] Postgres is the only planned truth/queue/event store; Valkey is cache/rate limits only.
- [ ] Judge has no public port and no main Docker socket.
- [ ] `migrations/` runs through the one-shot SQLx migration service.
- [ ] Rootless sandbox daemon boundary is documented in `deploy/`.
- [ ] MIT license and third-party notices exist.
- [ ] CI checks fmt, Clippy, tests, Svelte check/build, and cargo-deny licenses.
- [ ] All CI-equivalent checks are green.
