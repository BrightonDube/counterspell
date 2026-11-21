# Third-party notices

Counterspell is distributed under the MIT License in `LICENSE`.

The engine currently uses the following Rust crates. Their license metadata
was checked with `cargo deny check licenses` on 12 November 2026.

## Runtime dependencies

- **serde 1.0.229** — David Tolnay and contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/serde>
- **serde_json 1.0.154** — David Tolnay and contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/serde_json>

The runtime dependency tree also includes `serde_core 1.0.229`,
`serde_derive 1.0.229`, `proc-macro2 1.0.107`, `quote 1.0.47`, `syn 3.0.10`,
`unicode-ident 1.0.27`, `itoa 1.0.19`, `memchr 2.8.3`, and `zmij 1.0.23`.
The test dependency tree also includes `autocfg 1.5.0` and `libc 0.2.191`.
These crates retain their upstream MIT, Apache-2.0, or MIT/Apache-2.0
licensing as recorded in `Cargo.lock` and the crates.io package metadata.

- **uuid 1.29.2** — The uuid contributors — Apache-2.0 OR MIT —
  <https://crates.io/crates/uuid> (run-lifecycle T01, shared run ids)

## Development dependencies

- **proptest 1.12.0** — The proptest contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/proptest>

The reduced-feature proptest tree includes `bitflags`, `chacha20`, `cfg-if`,
`cpufeatures`, `rand_core`, `core_detect`, `num-traits`, `rand`, `getrandom`,
`rand_xorshift`, `regex-syntax`, and `unarray`. Their upstream license
metadata is checked by `cargo deny`; they are test-only dependencies and are
not linked into the engine's runtime dependency graph.

## Grimoire dependencies

The pack validator (`counterspell-grimoire`) adds these direct and tree
packages. `cargo deny check licenses` passed on them on 13 November 2026.

- **toml 0.8.23** — Ed Page and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/toml>

The `toml` tree adds `toml_datetime 0.6.11`, `toml_edit 0.22.27`,
`toml_write 0.1.2`, `serde_spanned 0.6.9`, `winnow 0.7.15`,
`indexmap 2.14.2`, `hashbrown 0.17.1` and `equivalent 1.0.3`. Their
upstream licences are recorded in `Cargo.lock` and the crates.io metadata.

## Notice policy

This file records direct engine dependencies and the dependency-tree packages
introduced by the engine manifest. Generated or upstream license text is not
replaced here; the corresponding package metadata and lockfile remain the
source for exact copyright and notice text.

## Judge dependencies (run-lifecycle T08)

The judge crate (`counterspell-judge`) adds these direct dependencies. `cargo deny check licenses` passed on them on 13 November 2026.

- **sqlx 0.8.6** — launchbadge and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/sqlx> (Postgres driver; TLS features off)
- **tokio 1.54.1** — Tokio contributors — MIT — <https://crates.io/crates/tokio>
- **tracing 0.1.44** — Tokio contributors — MIT — <https://crates.io/crates/tracing>
- **thiserror 2.0.21** — David Tolnay — MIT OR Apache-2.0 — <https://crates.io/crates/thiserror>

The judge also uses `serde_json`, `uuid` and `counterspell-common` as listed above.

Added by judge-sandbox T01 (14 November 2026; `cargo deny check licenses` passed):

- **bollard 0.21.1** — Bollard contributors — Apache-2.0 — <https://crates.io/crates/bollard> (Docker API client; optional, behind the `sandbox` feature; used only on the sandbox daemon)
- **tar 0.4.46** — tar-rs contributors — MIT OR Apache-2.0 — <https://crates.io/crates/tar> (builds the source archive delivered into a container)
- **toml 0.8.23** — toml-rs contributors — MIT OR Apache-2.0 — <https://crates.io/crates/toml> (reads `languages/languages.toml`)
- **bytes 1.12.1** — Tokio contributors — MIT — <https://crates.io/crates/bytes>
- **futures-util 0.3.34** — Rust async working group — MIT OR Apache-2.0 — <https://crates.io/crates/futures-util>
- **sha2 0.10.9** — RustCrypto developers — MIT OR Apache-2.0 — <https://crates.io/crates/sha2>
- **tempfile 3.27.0** (dev) — Steven Allen and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/tempfile>

The judge's `counterspell-engine` dependency is a workspace crate and carries the MIT licence of this repository.

## Server dependencies (run-lifecycle T20)

The server crate (`counterspell-server`) adds these direct dependencies. Licences are from the crates.io metadata in `Cargo.lock`. `sqlx`, `tracing` and `thiserror` are listed above under the judge.

- **redis 0.32.7** — redis-rs contributors — BSD-3-Clause — <https://crates.io/crates/redis> (Valkey client; `tokio-comp` and `script` features only, no TLS)
- **tracing-subscriber 0.3.23** — Tokio contributors — MIT — <https://crates.io/crates/tracing-subscriber> (`fmt` and `std` features only)
- **sha2 0.10.9** — RustCrypto developers — MIT OR Apache-2.0 — <https://crates.io/crates/sha2>

## Submission validation dependencies (run-lifecycle T22)

The server uses these for the parse check in submission validation (VAL-2). `cargo deny check licenses` passed on them on 13 November 2026.

- **tree-sitter 0.25.10** — tree-sitter contributors — MIT — <https://crates.io/crates/tree-sitter>
- **tree-sitter-python 0.25.0** — tree-sitter contributors — MIT — <https://crates.io/crates/tree-sitter-python>
- **tree-sitter-javascript 0.25.0** — tree-sitter contributors — MIT — <https://crates.io/crates/tree-sitter-javascript>
- **proptest 1.9** (server dev-dependency, `std` feature only) — MIT OR Apache-2.0 — <https://crates.io/crates/proptest>

## Auth dependencies (auth T01)

The auth module adds these direct dependencies to `counterspell-server`. `cargo deny check licenses` passed on them on 13 November 2026, with `reqwest` and `oauth2` left out (see the note below).

- **argon2 0.5.3** — RustCrypto developers — MIT OR Apache-2.0 — <https://crates.io/crates/argon2>
- **axum-extra 0.12.6** (feature `cookie`) — Axum contributors — MIT — <https://crates.io/crates/axum-extra>
- **hmac 0.12.1** — RustCrypto developers — MIT OR Apache-2.0 — <https://crates.io/crates/hmac>
- **rand 0.9.5** — The Rand Project developers — MIT OR Apache-2.0 — <https://crates.io/crates/rand>
- **base64 0.23.1** — Marshall Pierce and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/base64>
- **hex 0.4.3** — KokaKiwi and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/hex>
- **subtle 2.6.1** — isis lovecruft and Henry de Valence — BSD-3-Clause — <https://crates.io/crates/subtle>
- **url 2.5.8** — The rust-url developers — MIT OR Apache-2.0 — <https://crates.io/crates/url>
- **percent-encoding 2.3.2** — The rust-url developers — MIT OR Apache-2.0 — <https://crates.io/crates/percent-encoding>
- **async-trait 0.1.92** — David Tolnay — MIT OR Apache-2.0 — <https://crates.io/crates/async-trait>
- **time 0.3.55** — Time contributors — MIT OR Apache-2.0 — <https://crates.io/crates/time>
- **wiremock 0.6.5** (dev) — Kenton Varda and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/wiremock>
- **tower 0.5.3** (dev, feature `util`) — Tower contributors — MIT — <https://crates.io/crates/tower>
- **reqwest 0.12.28** (features `json`, `rustls-tls`, default features off) — Sean McArthur and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/reqwest>
- **oauth2 5.0.0** (default features off) — David Ramsay and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/oauth2>
- **webpki-roots 1.0.12** (transitive through `reqwest` `rustls-tls`; Mozilla root certificate list) — Chromium and webpki-roots contributors — CDLA-Permissive-2.0, approved by Brighton on 14 Nov 2026 for this crate only (`deny.toml` exception) — <https://crates.io/crates/webpki-roots>
- **ring 0.17.14** — Brian Smith and contributors — Apache-2.0 AND ISC — <https://crates.io/crates/ring>
- **data-encoding 2.11.1** — Ivan Maidanski and contributors — MIT — <https://crates.io/crates/data-encoding>
- **webauthn-rs, webauthn-rs-proto, webauthn-rs-core, webauthn-attestation-ca, base64urlsafedata 0.5.5** — Kanidm contributors — MPL-2.0 — <https://github.com/kanidm/webauthn-rs/tree/v0.5.5>
- **qrcode-generator 2.0.4** (web, authenticator enrollment) — Kazuhiko Arase — MIT — <https://www.npmjs.com/package/qrcode-generator/v/2.0.4>

The licence decision for `reqwest` and `oauth2` was approved on 14 Nov 2026 (`webpki-roots` option). The earlier "held back" note is superseded.

## Web editor dependencies (ide-workspace T07)

- **monaco-editor 0.57.0** — Microsoft Corporation — MIT — <https://www.npmjs.com/package/monaco-editor> (lazy-loaded editor; the Crypt theme is defined in `web/src/lib/editor/crypt-theme.ts`)
- **marked** (transitive, used by Monaco's hover markdown) — MIT — <https://www.npmjs.com/package/marked>
- **dompurify** (transitive, used by Monaco to sanitise hover markup) — dual-licensed MPL-2.0 OR Apache-2.0; used under Apache-2.0 — <https://www.npmjs.com/package/dompurify>
- **@types/trusted-types** (transitive type definitions) — MIT — <https://www.npmjs.com/package/@types/trusted-types>

## Efficiency Familiar web dependencies (efficiency-familiar T26)

- **gsap 3.15.1** — GreenSock (Webflow) — GSAP "Standard No Charge" licence (proprietary, commercial use allowed; accepted exception in `docs/licence-rules.md`). npm dependency only; the source is not vendored. Used only by the landing page and the Familiar. <https://www.npmjs.com/package/gsap> · <https://gsap.com/standard-license>

## Web font dependencies (ide-workspace T05)

Approved by Brighton in the standing decisions (14 Nov 2026). Fonts are self-hosted from the pinned packages and bundled at build time. No CDN is used.

- **geist 1.9.2** — Vercel — OFL-1.1 (Geist sans, `Geist-Variable.woff2`, loaded in `web/src/app.css`) — <https://github.com/vercel/geist-font/blob/main/LICENSE.txt>
- **@fontsource/jetbrains-mono 5.3.0** — JetBrains, packaged by Fontsource — OFL-1.1 (weights 400, 500, 600, 700, imported in `web/src/app.css`) — <https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt>
- **Creepster (Regular, Latin woff2)** — Font Diner, Inc (copyright 2011), SIL Open Font License 1.1, Reserved Font Name "Creepster" — self-hosted as `web/static/fonts/Creepster-Regular.woff2`, declared in `web/src/app.css`, used only for the landing title-screen wordmark and ENTER label (landing-steel-night T21). Full licence text: `web/static/fonts/creepster/OFL.txt`. <https://github.com/google/fonts/tree/main/ofl/creepster>

Added by run-lifecycle T29 (approved by Brighton in the standing decisions, 14 Nov 2026): `futures-util` is now a direct dependency of `counterspell-server`, used only to build the SSE event stream (`unfold`). The crate is already listed above.

Added by run-lifecycle T31 (test-only, server dev-dependencies; `cargo deny check licenses` passed):

- **hyper 1.12.0** — hyper contributors — MIT — <https://crates.io/crates/hyper> (HTTP/1 client that reads the SSE stream in tests)
- **hyper-util 0.1.21** — hyper contributors — MIT — <https://crates.io/crates/hyper-util> (Tokio I/O adapter for the test client)
- **http-body-util 0.1.5** — hyper contributors — MIT — <https://crates.io/crates/http-body-util> (body reading in the test client)

Added by the run-lifecycle close-out (T39). Both were direct server dependencies before this entry; the notices check found them missing:

- **axum 0.8.9** — Tokio contributors — MIT — <https://crates.io/crates/axum> (HTTP router, handlers and SSE responses for `counterspell-server`)
- **tower-http 0.6.11** — Tower contributors — MIT — <https://crates.io/crates/tower-http> (static files, body limit, response headers, compression in `counterspell-server`)

Added by dashboard-fsrs T10 (browser end-to-end tests; approved by Brighton on 16 Nov 2026; `web/` dev dependency only, never shipped):

- **@playwright/test 1.57.0** (with `playwright` and `playwright-core`) — Microsoft Corporation — Apache-2.0 — <https://www.npmjs.com/package/@playwright/test> (runs `web/e2e/*.spec.js` in Chromium against `counterspell-testserver`)


Added by break-room T01 (the break-room pages only; both pinned and loaded from first-party code):

- **chess.js 1.4.0** - Jeff Hlywa - BSD-2-Clause - <https://www.npmjs.com/package/chess.js> (move generation and legality for the daily puzzle and the chess bot)
- **cm-chessboard 8.15.8** - Stefan Haack - MIT - <https://www.npmjs.com/package/cm-chessboard> (the board UI on the break-room pages)

Added by dashboard-fsrs T11 (server dependency for the spaced-repetition scheduler, kept behind the reviews adapter):

- **fsrs 7.0.0** — Open Spaced Repetition — BSD-3-Clause — <https://crates.io/crates/fsrs>
