# Rust lesson: repo-skeleton T03

The workspace manifest gives every crate the same Rust edition, version, and lint rules.
A workspace keeps related packages buildable together while preserving their boundaries.
The six package names are explicit, so Cargo cannot silently add an alternate architecture.
The initial source files are intentionally tiny so later tasks can add one responsibility at a time.
# Rust lesson: repo-skeleton T04

Rust libraries can expose small constants and types without owning a process or runtime.
A crate boundary is useful documentation: `common` shares data, while `engine` stays pure.
Keeping dependencies out of these manifests prevents future sandbox concerns from leaking into them.
Doc comments become generated API documentation and explain the ownership boundary to reviewers.
# Rust lesson: repo-skeleton T05

A binary crate starts at `main`, while a library crate exposes reusable Rust APIs.
Grimoire keeps its library and `validate` command in one package, which avoids a second crate boundary.
An empty `main` is still a deliberate process boundary; it does not imply that untrusted code is safe to run.
Later async and trait work belongs in the judge and MCP specs, not in these placeholders.
# Rust lesson: repo-skeleton T06

No Rust code changed in T06; the frontend is intentionally independent of the server crate.
The server will consume the generated `web/build` directory later rather than starting npm at runtime.
Keeping the build artifact separate lets the Rust process remain the only production web server.
# Rust lesson: repo-skeleton T08

`Result` lets startup report configuration, bind, and server errors instead of hiding them.
The `?` operator returns an error early while preserving the normal success path.
`Router` owns route composition, and `Json<Value>` serializes the health response without hand-building JSON.
Tokio supplies the async runtime and listener, while Axum handles HTTP routing on top of it.
# Rust lesson: repo-skeleton T09

`Path` borrows a filesystem path, while `PathBuf` owns one and can be moved into the router.
The server validates the directory before starting, so a missing build fails as a startup error.
Axum registers API routes before the broad static fallback; the explicit API 404 keeps unknown API paths JSON.
`ServeFile` provides the SPA entry point only after `ServeDir` cannot find a static asset.
# Rust lesson: repo-skeleton T10

No Rust code changed in T10; the migration is intentionally independent of the Rust workspace.
SQL migrations are versioned files, so SQLx can apply them once and record their history.
The bootstrap table is metadata only; future specs will add users, jobs, and run events to Postgres.
# Rust lesson: repo-skeleton T12

The Dockerfile compiles Rust in a builder stage and copies only release binaries into runtime stages.
This keeps compiler tooling out of production images and makes the server image smaller.
The server runtime copies static files but no Docker client or socket, preserving the architecture boundary.
The migration image pins the SQLx CLI version separately from application dependencies.
# Rust lesson: repo-skeleton T15

No Rust source changed in T15; CI protects the Rust workspace with formatting, Clippy, and tests.
`cargo-deny` checks dependency license metadata against an explicit allow-list.
Running these checks on every push makes small Rust changes easier to review than a late failure.
# Rust lesson: repo-skeleton T15 follow-up

Clippy caught a needless ownership transfer: `parse_port` only reads its input.
Changing `String` to `&str` lets the function borrow the environment value without taking ownership.
Borrowing is useful for small validation helpers because the caller keeps its string and no allocation is needed.
