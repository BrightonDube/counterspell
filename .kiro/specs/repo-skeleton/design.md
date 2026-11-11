# Low-level design: repo-skeleton

## Design approach

This is a Design-First, Low-Level feature. The architecture is fixed by `docs/system-design.md`; this design turns the canonical layout and first milestone into an implementation-ready skeleton. It creates only the repository, build, serving, local infrastructure, migration, deployment-note, licensing, and CI foundations.

The implementation must keep untrusted code out of the server and developer host. The server is a static/API process and must never receive a Docker socket. The future trusted judge is the only component allowed to talk to the separate rootless sandbox daemon.

## Scope and non-goals

### Included

- One Cargo workspace with exactly six packages.
- Minimal compiling entry points for the six packages.
- An Axum server with `GET /api/health`, static-file serving, and SPA fallback.
- A SvelteKit 5 static build in `web/` with Tailwind 4 semantic Steel Night tokens.
- A multi-stage application image and Compose services for Caddy, server, judge, Postgres, Valkey, and migration execution.
- A minimal SQLx migration path.
- Documentation for the separate rootless sandbox daemon.
- GitHub Actions checks, MIT licensing, and third-party notice scaffolding.

### Deferred

Authentication, OAuth, sessions, full API routes, Postgres domain schema, queue claiming, leases, run events, SSE, judge execution, Docker/Bollard integration, language drivers, ghost generation, pack content, MCP tools, hints, dashboard, Familiar, landing visuals, and IDE work are later specs.

## Canonical repository map

```text
counterspell/
├── Cargo.toml                         # workspace; exactly six members
├── Cargo.lock                         # committed application lockfile
├── Dockerfile                         # multi-stage web/Rust/migration image
├── LICENSE                            # MIT
├── THIRD-PARTY-NOTICES.md
├── README.md
├── docker-compose.yml
├── Caddyfile
├── .github/workflows/ci.yml
├── crates/
│   ├── server/                        # package counterspell-server, binary
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── judge/                         # package counterspell-judge, binary placeholder
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── engine/                        # package counterspell-engine, library
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── grimoire/                      # package counterspell-grimoire, library + validate binary
│   │   ├── Cargo.toml
│   │   └── src/{lib.rs,main.rs}
│   ├── mcp/                           # package counterspell-mcp, binary placeholder
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── common/                        # package counterspell-common, library
│       ├── Cargo.toml
│       └── src/lib.rs
├── web/                               # SvelteKit static app
├── migrations/                        # SQLx migration files
├── languages/                         # reserved canonical language config location
├── packs/                             # reserved canonical Grimoire pack location
└── deploy/                            # rootless sandbox daemon notes
```

There are six Cargo packages, not six independent architecture variants. `counterspell-grimoire` may expose a library and its `validate` binary within one package; this does not create another crate name. The judge, MCP, and server are executable packages. Engine and common are libraries.

## Cargo package boundaries

| Package | Kind | Skeleton responsibility | Boundary |
|---|---|---|---|
| `counterspell-server` | binary | Tokio/Axum process, health route, static files | May depend on `counterspell-common`; must not depend on Bollard or access Docker. |
| `counterspell-judge` | binary | Placeholder trusted worker entry point | Later owns queue, verdicts, and sandbox control. No public port. Only later receives the sandbox socket. |
| `counterspell-engine` | library | Empty/pure library baseline | No process execution, filesystem sandboxing, or Docker dependency. |
| `counterspell-grimoire` | library + binary | Placeholder pack model and `validate` command | Later owns pack parsing/validation; no user-code execution in this skeleton. |
| `counterspell-mcp` | binary | Placeholder rmcp stdio process boundary | Later delegates code-running work to the trusted sandbox path; never runs code directly on the developer host. |
| `counterspell-common` | library | Shared baseline types module | Contains only types that genuinely cross server/judge boundaries; no Axum or Docker dependencies. |

Dependencies should be intentionally small. The initial server needs Axum, Tokio, and tower-http's filesystem service. The frontend is built separately by npm. Do not add database, Valkey, Bollard, OAuth, LLM, or MCP implementation dependencies merely to make placeholders appear complete. Add those in the feature specs that need them.

## Server design

### Configuration

`counterspell-server` reads non-secret runtime configuration from environment variables, with safe local defaults for the skeleton:

- `HOST` default `0.0.0.0`.
- `PORT` default `8080`.
- `STATIC_DIR` default `web/build` for local execution and `/app/web/build` in the runtime image.

The server should return a useful startup error when address binding fails. It should propagate filesystem and listener errors with context instead of using `unwrap()` or `expect()` in production code.

### Route precedence

The Axum router is assembled in this order:

1. `GET /api/health` returns a small JSON object such as `{ "status": "ok" }`.
2. A future `/api` namespace remains reserved for API routes and must be registered before the fallback.
3. Static assets are served from `STATIC_DIR` with `ServeDir`.
4. The final non-API fallback serves `STATIC_DIR/index.html` for SvelteKit client-side routes.

The fallback must not turn an unknown API route into an HTML page. `/api/*` routes that are not implemented should remain API errors in the server implementation, rather than being handled by the frontend fallback.

The health response is intentionally independent of Postgres and Valkey in this skeleton. Readiness aliases are a follow-up decision recorded below.

### Static build paths

The SvelteKit adapter-static output is `web/build`. In the Docker runtime image it is copied to `/app/web/build`, and `STATIC_DIR=/app/web/build` is set for the server. Local `cargo run -p counterspell-server` can serve a locally generated `web/build` using its default. The server must not invoke npm at runtime.

## SvelteKit and Tailwind design

`web/` uses SvelteKit 5 with the static adapter. The minimal page is a real Svelte route and contains a short Counterspell welcome message plus a visible build marker useful for the Compose smoke test.

Tailwind 4 is imported using the CSS-first configuration. `web/src/app.css` defines the semantic `@theme` names and the dark `:root[data-theme="dark"]` variables from `docs/palette.md`:

- base: `bg`, `surface`, `raised`, `border`, `border-strong`;
- text: `text`, `text-muted`, `text-subtle`;
- accents: `accent-moss*`, `accent-candle*`, `info`, `focus`;
- semantic states: `success*` and `danger*`.

Components use semantic classes and do not embed raw hex values. Bright green is reserved for success and red for failures. The initial page is deliberately calm; landing GSAP, Monaco, and IDE work are deferred. Bits UI is included only to establish the chosen frontend direction where the current SvelteKit/Tailwind setup requires it; no component suite is built in this skeleton.

## Docker build flow

The root `Dockerfile` uses named stages so the same build definition can produce application and migration images:

1. A Node build stage installs the locked web dependencies and runs the SvelteKit static build.
2. A Rust build stage copies the workspace and builds the release binaries for the six packages.
3. A migration tooling stage provides `sqlx migrate run` and the migration directory. The chosen image/tooling must be pinned in the implementation and documented if it is external.
4. A small server runtime stage copies the `counterspell-server` binary and `/app/web/build` and runs as a non-root runtime user.
5. A judge runtime stage contains only the placeholder judge binary and its configuration. It does not include the main Docker socket.
6. A migration runtime stage contains the SQLx CLI and `/app/migrations`.

The Compose `server` service builds the server target, `judge` builds the judge target, and `migrate` builds or references the migration target. The Caddy service remains the edge proxy and does not serve the frontend directly.

## Compose topology

The main Compose environment runs on the primary Docker daemon:

```text
browser :80/:443 → caddy → server:8080
                                  ├── postgres:5432
                                  └── valkey:6379
migrate ───────────────→ postgres:5432 (one shot)
judge ────────────────→ postgres:5432 (internal only; no public port)
```

- `caddy` publishes `80:80` and `443:443`, has `caddy_data` and `caddy_config` volumes, and proxies to `server:8080`.
- `server` has no Docker socket mount. It receives only the application configuration needed for this skeleton and future database/cache URLs as placeholders.
- `postgres:17` uses `pgdata`, a health check, and an internal network.
- `valkey/valkey:8` uses an internal network and no persistence requirement for cache/rate-limit data.
- `judge` has no `ports:` section and no main daemon socket mount. Its database access is a future limited role; the skeleton must not grant it OAuth, Resend, LLM, or Docker credentials.
- `migrate` depends on the Postgres health check, runs once, and is not a long-running API service.

For local HTTP smoke testing, Caddy listens on port 80 and can proxy to the server without requiring production certificates. Production HTTPS configuration and domain-specific TLS behavior remain deployment configuration.

Compose must make service dependencies explicit, but dependency order alone is not readiness. Health checks and clear process failures are required. The initial migration is intentionally minimal, and the skeleton does not require the sandbox daemon to serve the hello page.

## Deployment boundary note

`deploy/rootless-sandbox-daemon.md` documents the separate sandbox daemon required by later judge work:

- an unprivileged `sandbox` OS user owns a systemd user service;
- the daemon listens on its own rootless socket, separate from the main Compose daemon;
- sandbox images are pre-pulled and are not pulled at run time;
- rootless Docker requires cgroup v2 delegation to make CPU, memory, and PID limits effective, which must be verified on the target VPS;
- only the trusted judge may later receive the sandbox socket;
- the server, Caddy, Postgres, Valkey, and secrets remain on the main daemon and never receive that socket;
- no untrusted code is run as part of the repo skeleton;
- `JUDGE_CONCURRENCY=2` is the initial operational default.

The note is documentation, not permission to mount a host Docker socket into the server or to implement the sandbox early.

## SQLx migration design

The first migration file uses SQLx's timestamped naming convention and contains only a minimal valid bootstrap change, such as a schema-level metadata table needed to prove migration execution. It must be safe to run once and safe for the one-shot Compose service to report success on subsequent starts. Users, submissions, jobs, run events, and all other domain tables belong to later migrations.

The implementation must make the migration directory explicit through `SQLX_OFFLINE`/runtime configuration only where appropriate. The Compose migration command is `sqlx migrate run --source /app/migrations` with a database URL supplied through environment configuration. Secrets are not committed and are not read or written by this spec.

## CI design

`.github/workflows/ci.yml` runs on pushes, and may run on pull requests as a compatible extension. Its jobs may be split for speed, but every required check must run:

- Rust: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace`.
- Web: from `web/`, `npm ci`, `npm run check`, and `npm run build`.
- Licenses: `cargo deny check licenses` from the repository root.

The workflow installs stable Rust, Node.js, npm dependencies from the lockfile, and cargo-deny. If the initial server has no database-dependent tests, `cargo test --workspace` must still pass without Postgres. Later server tests may add a Postgres service or testcontainers as their own spec requires.

## Verification and failure behavior

The implementation is verified in layers:

1. `cargo fmt --all -- --check` confirms formatting.
2. `cargo clippy ... -D warnings` confirms warning-free Rust.
3. `cargo test --workspace` confirms all six packages compile and test.
4. `cd web && npm run check && npm run build` confirms the static frontend.
5. `cargo deny check licenses` confirms the dependency policy.
6. `docker compose config` confirms the service graph and mounts.
7. `docker compose up` starts the stack. A smoke request to `http://localhost/` must contain the Svelte build marker, and `http://localhost/api/health` must return the health JSON.
8. The smoke test must establish that the response came through Caddy and the Axum static route, not a frontend dev server. The server image must contain the built static files, and no web dev server is exposed by Compose.

If Postgres or Valkey is unavailable, their health checks and dependent service logs must make the issue visible. If migration execution fails, `migrate` must exit non-zero. If the web or Rust build fails, the image build must fail rather than produce a partial runtime image. No readiness endpoint may claim the full production system is ready before its dependencies are actually wired.

## Security and architecture invariants

- No `.env`, key, credential, or secret file is created or read by this spec.
- No service gets the main `/var/run/docker.sock`.
- The server has no Docker client dependency and no Docker socket.
- Valkey is not used for queueing or event delivery.
- The future judge remains the trusted verdict owner and the only component that can use the separate sandbox daemon.
- Static page, stdout, stderr, and future test output are treated as data; this skeleton does not render program output as HTML.
- Production Rust code avoids `unwrap()` and `expect()`; errors retain context at process and filesystem boundaries.

## Open questions

1. **Health route aliases.** The kickoff scope explicitly requires `/api/health`, while the broader system design names operational `/healthz` and `/readyz` routes. **Assumed choice:** implement only `/api/health` in this skeleton and reserve `/healthz` and `/readyz` for a later operations/API spec.
2. **Initial migration contents.** The architecture does not mandate a first table. **Assumed choice:** create one minimal valid bootstrap migration so the Compose migration service is testable, while deferring all domain tables to later specs.
3. **Rootless socket in local Compose.** The deployment needs a separate rootless sandbox daemon, but the hello-page stack does not execute code. **Assumed choice:** document and preserve the boundary without requiring or mounting the sandbox socket in local Compose.
4. **Frontend visual depth.** The milestone calls for a served Svelte page, while the full Steel Night landing and IDE are later work. **Assumed choice:** implement semantic palette scaffolding and a minimal page only; defer cinematic visuals, Monaco, and Familiar work.
5. **Migration image/tooling.** The system design specifies `sqlx migrate run` but not a particular image. **Assumed choice:** use a dedicated migration target in the project Dockerfile, with the SQLx CLI version pinned alongside the workspace tooling, rather than relying on an unpinned third-party runtime image.
6. **Caddy local TLS.** Production uses automatic HTTPS, but local `docker compose up` must work at localhost without certificates. **Assumed choice:** use plain HTTP on local port 80 and keep production domain/TLS settings in deployment configuration.
