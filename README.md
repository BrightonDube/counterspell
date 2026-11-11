# Counterspell

Counterspell is a horror-themed coding practice platform. When a solution
fails, the future judge will find and shrink a failing input into a minimized
counterexample and explain the bug.

This repository is the foundation skeleton. It contains the canonical Rust
workspace, a small Axum server, a SvelteKit static page, local Compose
infrastructure, and the migration boundary. Authentication, judging, ghost
hunting, language drivers, and the sandbox execution path are later features;
this skeleton does not run submitted code.

## Repository layout

- `crates/server/` — `counterspell-server`, Axum API and static files.
- `crates/judge/` — `counterspell-judge`, future trusted worker boundary.
- `crates/engine/` — `counterspell-engine`, future pure ghost engine.
- `crates/grimoire/` — `counterspell-grimoire`, future pack model and validator.
- `crates/mcp/` — `counterspell-mcp`, future MCP stdio boundary.
- `crates/common/` — `counterspell-common`, shared server/judge types.
- `web/` — SvelteKit static frontend.
- `languages/` — reserved language configuration location.
- `packs/` — reserved Grimoire pack location.
- `migrations/` — SQLx migrations.
- `deploy/` — deployment and sandbox-boundary notes.

Postgres is the planned source of truth, queue, and event store. Valkey is
reserved for cache and rate limits only; it is not a queue or event bus.

## Local development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

cd web
npm ci
npm run check
npm run build
```

Run the server locally after building the frontend:

```bash
cargo run -p counterspell-server
curl http://localhost:8080/api/health
```

The server uses `HOST=0.0.0.0`, `PORT=8080`, and `STATIC_DIR=web/build` by
default. Configuration is supplied through the process environment. Do not
commit secrets or `.env` files.

## Compose smoke test

With Docker Engine and Compose available:

```bash
docker compose up --build
curl http://localhost/
curl http://localhost/api/health
```

Only Caddy publishes ports. The server has no Docker socket, the judge has no
public port, and the local skeleton does not mount a sandbox socket. The
separate rootless sandbox daemon is documented in
`deploy/rootless-sandbox-daemon.md` for a later judge implementation.

## License

Counterspell is MIT licensed. Dependency and asset notices are tracked in
`THIRD-PARTY-NOTICES.md` as they are introduced and verified.
