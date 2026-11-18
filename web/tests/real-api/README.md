# Real-API browser harness (ide-workspace T24)

`real-api-check.mjs` drives the **built** Svelte output served by the real Axum
server, with the trusted judge claiming runs and executing them inside the
rootless sandbox daemon. Nothing is intercepted: no `page.route`, no fixture
route, no mocked run responses. The only development stub in the path is the
log mail transport (`APP_ENV=development`), which prints the magic link to the
server log instead of sending email.

## Stack bring-up

Everything runs locally; the sandbox daemon must be the rootless one at
`/run/user/1000/docker.sock`, never the host Docker socket.

1. Dev services: `cs-dev-pg` (Postgres at `127.0.0.1:55432`) and
   `cs-dev-valkey` (`127.0.0.1:56379`):
   `docker start cs-dev-pg cs-dev-valkey`.
2. Build the web bundle: `cd web && npm run build` (Axum serves `web/build`
   with SPA fallback — no Vite dev server is involved).
3. Build binaries: `cargo build -p counterspell-server -p counterspell-judge
   --features sandbox` (the judge's `sandbox` feature is what connects the
   rootless daemon; the binary loads problems through `PgProblemTests`).
4. Materialize the private half of the `add-two` pack outside the repository
   and stamp its approval (`node tests/real-api/stamp-approval.mjs <scratch>`),
   then seed: `counterspell-catalogue seed-demo --public-root packs
   --private-root <scratch>/private` from the repo root. The importer rejects a
   private root nested inside or adjacent to the public tree.
5. Start the server with stderr teed to `.scratch/server.log` (the harness
   scrapes the dev mail link from it):
   `HOST=127.0.0.1 PORT=8088 STATIC_DIR=web/build DATABASE_URL=postgres://…
   VALKEY_URL=redis://127.0.0.1:56379 APP_ENV=development
   PUBLIC_BASE_URL=http://127.0.0.1:8088 SESSION_SECRET=<32+ bytes>
   HINT_PROVIDERS= RUST_LOG=info ./target/debug/counterspell-server
   2>web/tests/real-api/.scratch/server.log`
   `HINT_PROVIDERS=` must be set explicitly — the default requires AI keys.
6. Start exactly one judge — and no other queue consumer. A leftover
   `counterspell-testserver` shares the same `jobs` table and answers runs with
   scripted results:
   `DATABASE_URL=postgres://… LANGUAGES_FILE=languages/languages.toml
   SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock
   ./target/debug/counterspell-judge`

## Browser installation

The harness uses `playwright-core` (Apache-2.0) as a driver only — it does not
download browsers. It launches an installed Chrome/Edge through
`channel: 'chrome'`, so a system Chrome is the only prerequisite. On this
machine node runs Windows-side against the WSL-hosted servers; localhost
reaches them either way.

## Run

`node web/tests/real-api/real-api-check.mjs`

The check signs in through the real magic-link flow (the token stays in the URL
fragment until the user confirms), opens `/problems/add-two`, types a correct
solution into the real Monaco editor, runs samples, then submits a wrong
solution. It asserts real terminal verdicts, reads the authoritative terminal
model, and verifies in Postgres that events persisted and that hidden-test
events never carry input, expected or got fields.

Runtime artifacts (server/judge logs, probes) live in `.scratch/`, which is
git-ignored.
