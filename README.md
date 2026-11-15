# Counterspell

[![CI](https://github.com/BrightonDube/counterspell/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/BrightonDube/counterspell/actions/workflows/ci.yml?query=branch%3Amain)
[![Security](https://github.com/BrightonDube/counterspell/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/BrightonDube/counterspell/actions/workflows/security.yml?query=branch%3Amain)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/BrightonDube/counterspell/badge)](https://scorecard.dev/viewer/?uri=github.com/BrightonDube/counterspell)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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

## Run lifecycle

A run moves from `POST /api/runs` to one terminal result, and Postgres is the only store for it.

- The server saves the submission and its queue job in one transaction. An `Idempotency-Key` returns the same run on a retry.
- A judge claims the job with `FOR UPDATE SKIP LOCKED` and a renewable lease. Every judge write must pass the claim's fence (job, owner and attempt).
- Each event is a row in `run_events` with a gap-free sequence number. The terminal result is written once.
- `GET /api/runs/{id}/events` streams the events as Server-Sent Events. The `id:` is the sequence number, so a reconnect with `Last-Event-ID` resumes where it stopped.
- Valkey holds rate-limit counters only. It is not a queue or an event bus.

Database tests use the `pg_` (and `valkey_`) prefix and are ignored by default. To run them, start a throwaway Postgres 17 and export its URL in your shell. Do not write the URL or password to a file:

```bash
docker run --rm -d --name cs-pg -e POSTGRES_PASSWORD=<throwaway> -p 5432:5432 postgres:17-alpine
export DATABASE_URL=postgres://postgres:<throwaway>@127.0.0.1:5432/postgres
export VALKEY_URL=redis://127.0.0.1:6379
cargo test --workspace --all-features -- --ignored pg_ valkey_
docker stop cs-pg
```

The definition-of-done test writes `target/run-lifecycle/dod-report.json`. Results from one recorded run are in `docs/kiro-log/run-lifecycle-dod.md`.

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

### Judge worker

`counterspell-judge` claims queued runs from Postgres and records their outcome.
Until the sandbox runner exists, it runs a placeholder executor, so every run ends
as an internal error after its retries. It never reports a verdict. Settings are
read from the environment (`JUDGE_CONCURRENCY`, `LEASE_SECS`, `RENEW_SECS`, and
the others in `crates/judge/src/config.rs`). It stops on `SIGINT` or `SIGTERM`.

## Sign-in and accounts

Users sign in with GitHub, Google, a magic link sent by email, or an email and password. Provider accounts are never matched to an existing account by email address; a signed-in user links a provider from the account settings. The server reads the auth settings (`SESSION_SECRET`, `PUBLIC_BASE_URL`, the OAuth client values, `RESEND_API_KEY` and the demo account variables) from its environment only, and never from `judge`. `deploy/auth-setup.md` lists the OAuth callback URLs and the demo account setup. Local development uses `APP_ENV=development` with the `log` mail transport, so magic links appear in the server log.

## License

Counterspell is MIT licensed. Dependency and asset notices are tracked in
`THIRD-PARTY-NOTICES.md` as they are introduced and verified.

## CI/CD and security

Every PR and push runs tests, coverage, linting, image builds and a security gate
(secrets, SAST, dependency audits, workflow lint). Images are signed and attested.
See [docs/ci-cd.md](docs/ci-cd.md) and [SECURITY.md](SECURITY.md).
