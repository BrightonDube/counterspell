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
# Rust lesson: repo-skeleton T01

No Rust source changed in T01; the license covers the Rust packages that follow.
A clear license lets future Rust code be reused and reviewed under the intended terms.

# Rust lesson: repo-skeleton T02

No Rust source changed in T02; the README documents where each Rust responsibility belongs.
A written crate map helps prevent accidental ownership overlap as the workspace grows.

# Rust lesson: repo-skeleton T07

No Rust source changed in T07; semantic frontend tokens are consumed by the future Rust-served page.
The Rust server stays responsible for delivery, while visual values remain in the frontend stylesheet.

# Rust lesson: repo-skeleton T11

No Rust source changed in T11; the deployment note protects the future judge boundary.
The judge will own sandbox access later, while the server remains a normal HTTP process.

# Rust lesson: repo-skeleton T13

No Rust source changed in T13; Compose connects services without adding Docker dependencies to Rust.
The server and judge remain separate binaries even though Compose starts both images.

# Rust lesson: repo-skeleton T14

No Rust source changed in T14; SQLx runs migrations as a one-shot operational process.
The application can later use the resulting Postgres schema without placing migration logic in the server binary.

# Rust lesson: repo-skeleton T16

No Rust source changed in T16; the image copies the already-built Rust binary into a non-root runtime.
The Rust server still serves the static build and never starts the frontend toolchain.

# Rust lesson: repo-skeleton T17

No Rust source changed in T17; the smoke test verified the Rust server behind Caddy.
The HTTP response path proves the static frontend is delivered by Axum rather than a development server.

# Rust lesson: repo-skeleton T18

No Rust source changed in T18; the final checks validate the entire Rust workspace.
Formatting, Clippy, tests, and the exact package metadata are release gates for later Rust features.

# Rust lesson: ghost-engine T01

Cargo features let a crate opt into only the parts it needs, such as Serde's derive macros.
Runtime dependencies stay separate from dev-dependencies, so proptest is available only to tests.
A module declaration creates a named boundary, while a small crate root keeps the public API easy to review.
Keeping the engine's dependency list short protects its pure, synchronous design.

# Rust lesson: ghost-engine T02

u64 keeps the PRNG state stable across platforms, while i64 is used only at the input boundary.
Wrapping arithmetic gives SplitMix64 its defined sequence without overflowing or panicking.
Rejection sampling avoids modulo bias when an integer range does not divide the random domain.
Length-prefixed hashing prevents different field boundaries from producing the same derived seed.

# Rust lesson: ghost-engine T03

Rust enums let one schema type represent several validated shapes while keeping each variant's fields explicit.
Serde attributes such as `tag`, `rename_all`, `default`, and `deny_unknown_fields` control the JSON format without custom parsing code.
A hand-written `Display` implementation turns a private error message into clear text for callers.

# Rust lesson: ghost-engine T04

A borrowed `&Value` lets validation inspect untrusted JSON without taking ownership of it.
`match` keeps each schema kind's shape checks separate, while `Result` returns the first useful violation.
A JSON Pointer path records where a nested value failed, so callers do not need to guess.
A HashSet makes uniqueness checking linear instead of repeatedly scanning the entire prefix.

# Rust lesson: ghost-engine T05

`Ordering` gives the shrinker one shared comparison rule instead of scattered size decisions.
`u128` safely holds large magnitude sums, while Serde stores it as text because JSON numbers are narrower.
Borrowing `&Value` keeps size calculations read-only and avoids moving input data.
A canonical JSON tie-break makes distinct values comparable even when their numeric metrics match.

# Rust lesson: ghost-engine T06

Serde attributes define stable wire names without changing Rust's enum names.
Borrowing JSON values lets Exact compare strict types without converting them first.
Sorting canonical element text gives Unordered multiset behavior while preserving duplicate counts.
A focused comparator keeps verdict rules reusable by the judge and ghost hunt.

# Rust lesson: ghost-engine T07

Enums make invalid classification states explicit, so every reference and user outcome gets a handled branch.
`Option<String>` records that an exception class belongs only to crash predicates.
Sanitizing at the engine boundary keeps untrusted text bounded before it reaches traces or persisted ghosts.
Serde names pin the stored vocabulary separately from the Rust type names.
# Rust lesson: ghost-engine T08

A function that takes `&mut SplitMix64` shows that generation changes the random state, so the same seed always replays the same values.
`checked_div` returns `None` for a zero divisor, which replaces a manual zero test with a single `map_or`.
The generator builds values from the schema without validating them, and the proptest checks them with `contains`, so a generator bug fails a test instead of being hidden.

# Rust lesson: ghost-engine T09

An `Iterator` produces one item per `next()` call, so the stream only builds candidates that the hunt actually asks for.
`VecDeque` gives cheap removal from the front, which is how the known ghosts and samples are handed out in order.
The odometer walks combinations like a car mileage counter: the last digit moves first and carries into earlier ones.
`checked_div`-style helpers and `Option` returns let an exhausted stage end with `None` instead of looping forever.

# Rust lesson: ghost-engine T10

A JSON Pointer such as `/0/3` names one position inside a value, so a move changes exactly that item and leaves the rest alone.
`value.pointer_mut(path)` returns `Option<&mut Value>`, so a missing path is handled with `if let` instead of a panic.
`&dyn Fn(...)` lets one function take any closure, which keeps the whole-value moves short.
A `match` on a tuple of `(schema, value)` handles every pairing, and the `_ => {}` arm makes mismatched shapes harmless.

# Rust lesson: ghost-engine T11

`div_ceil` gives the rounded-up half of a length without floating-point numbers, which keeps the chunk sizes exact.
`Iterator::chain` joins several lists of proposals without copying them into one new list.
`std::iter::once(()).flat_map(...)` postpones building each pass until the caller asks for it.
`get_mut(index)` returns an `Option`, so a bad index is skipped instead of crashing the program.

# Rust lesson: ghost-engine T12

A `&mut dyn FnMut(&Value) -> Probe` parameter lets the caller supply a yes-or-no test without the shrinker knowing what it does.
The loop finds the proposals first, then returns from a helper, so the borrow of `current` ends before `current` is replaced.
`VecDeque` keeps the last few trace entries cheaply, and `pop_front` drops the oldest once the cap is reached.
`u32::try_from(len)` turns a `usize` count into a `u32` with a visible fallback instead of an unchecked `as` cast.

# Rust lesson: ghost-engine T13

A trait such as `Oracle` is a promise about methods. The hunt will hold `&mut dyn Oracle`, so the judge and the tests can supply different implementations.
`Cell<Duration>` lets `ManualClock::advance` take `&self`, so a clock can be shared by reference while it moves.
`Result<(), &str>` from `validate` lists every rule the budget breaks as a short reason, and `?` or `match` makes the caller handle it.
Serde `with` attributes store `Duration` as plain milliseconds, which keeps the JSON readable for the judge.

# Rust lesson: ghost-engine T14

A recursive walk can carry its path as a `&str` argument, so each level builds only the path it needs.
`matches!(got, ProgramResult::Returned(value) if value == expected)` checks an enum variant and a condition in one line.
`map_or_else` supplies a fallback when an `Option` is empty, with no `unwrap()`.
Cutting text at a character boundary, not a byte index, means a multi-byte letter can never be split and produce invalid UTF-8.

# Rust lesson: ghost-engine T15

`Option<&T>` compared with `==` tells you whether two optional values match without cloning either one.
`let ... else` runs a fallback and returns when a pattern does not match, which keeps the happy path flat.
`Duration::saturating_sub` returns zero instead of panicking when the subtraction would go below zero.
A request-validation function that returns `Option<String>` lets the hunt refuse bad input with a reason, not a panic.

# Rust lesson: ghost-engine T16

A closure that borrows `oracle` and `stats` mutably is passed to `shrink` as `&mut dyn FnMut`, and the borrow ends when its block closes, so the code can read the results afterwards.
`Option::take` moves a value out of an `Option` and leaves `None`, which lets a loop compare the new run with the previous one without holding two borrows.
`match` guards with a moved binding (`Some(previous) if previous != run`) keep the comparison next to the branch that uses it.
The confirm step returns a small enum (`Confirmed`, `Unreproduced`, `OutOfTime`), so each outcome has its own explicit branch and none can be ignored by accident.

# Rust lesson: ghost-engine T17-T19

No Rust changed in these tasks. They added the pack JSON, Markdown and Python text files only.

# Rust lesson: ghost-engine T20

Integration tests in `tests/` each compile as their own crate, so shared code goes in `tests/support/mod.rs` and is pulled in with `mod support;`.
A function pointer type (`type Model = fn(&Value) -> ProgramResult;`) lets a `match` pick a model and pass it around without boxing.
`Option::get` and `.get(i)` on `serde_json::Value` return `None` instead of panicking, so a missing key in a fixture becomes a clear test failure.
The `let ... else` form (`let Some(x) = ... else { return ... };`) keeps the happy path flat when a missing value should exit early.

# Rust lesson: ghost-engine T21

`proptest!` runs each property on many random inputs, and when one fails it shrinks that input and saves the seed in a regression file so the case runs again first next time.
`prop_oneof!` with weights picks between strategies, and `prop_filter` drops values that fail a check, such as an invalid schema.
`prop_assert!` and `prop_assert_eq!` return an error instead of panicking, which lets proptest shrink the failing case.
A property can pass for the wrong reason, so a separate non-vacuity test checks that the hunt really finds a ghost for at least one oracle.

# Rust lesson: ghost-engine T22

`let ... else` checks a shape and exits early, so a test can name the wrong shape in its panic message.
Struct-update syntax (`HuntRequest { samples, ..HuntRequest::new(...) }`) sets fields when the value is built, which avoids the `clone_from` lint that clippy raises on `x.field = y.clone()`.
`assert_eq!` on `Option`s and enums compares whole values, so a test that checks the expected value, the got value and the kind all at once fails with a readable diff.
A mutation check (change one model, run the suite, restore the model) proves that the definition-of-done tests can fail.

# Rust lesson: ghost-engine T23

`std::fs::read_to_string` and `read_dir` read files from a test, so a test can check the source text of the crate.
`Path::new(env!("CARGO_MANIFEST_DIR"))` gives the crate folder at compile time, so the test does not depend on the directory it runs from.
`str::lines`, `trim_start`, `contains` and `split_once` are enough to scan text without a regex crate.
A guard earns trust only when it fails on a real violation, so this task added a forbidden call, saw the test fail, and removed it again.

# Rust lesson: ghost-engine T24

Tests that share a name prefix can be run as a group, so `cargo test --test properties wording` runs only the wording checks.
`serde_json::to_value` turns a Rust enum into JSON, and `assert_eq!` against a string literal checks the exact field contract that the database columns rely on.
`build_autopsy` is the public entry point, so the template wording is tested through the same path the judge will use.
A mutation check added a forbidden word to one template, saw the wording test fail, and then restored the file.

# Rust lesson: ghost-engine T25

Doc comments can link to items with square brackets, such as `[`hunt()`]`, and `cargo doc` checks that each link resolves.
A doctest is a code example in a doc comment that `cargo test --doc` compiles and runs, so the example in the crate docs cannot go stale.
`RUSTDOCFLAGS="-D warnings" cargo doc` turns a broken link into a build failure.
Documentation that shows a fake judge (a struct that implements `Oracle`) explains the seam without needing a sandbox.

# Rust lesson: ghost-engine T26

`cargo fmt --check`, `clippy -D warnings`, the test suite and the licence check are cheap, and each one catches a different kind of break.
Clippy's `-D warnings` turns lints into errors, so a style issue such as a missing backtick fails the build instead of waiting in a log.
A lint can also flag test code, so the same gate runs with `--all-targets` to include tests and examples.
Running the gate on the final code, after the last edit, is the only evidence that counts.

# Rust lesson: grimoire-packs T02

An `enum` such as `Severity` lists every allowed choice, so a typo like `Severiy::Eror` is a compile error, not a runtime surprise.
Each diagnostic code is a `const` built by a `const fn`, so a misspelt code fails to compile and the table exists before the program runs.
`#[derive(Serialize)]` with `rename_all = "lowercase"` turns the enum into the JSON words `error` and `warning` without hand-written code.
Tests import the code table with an explicit `use crate::codes;`, so production code does not carry an import that only tests use.

# Rust lesson: grimoire-packs T03

`Result<T, E>` makes a failure part of the return type, and `?` passes it up to the caller. The loader returns `io::Result` only when the pack directory itself cannot be read, so a bad file becomes a diagnostic instead of a crash.
Every size check uses `metadata().len()` before `fs::read`, so an oversize file is never loaded into memory.
`symlink_metadata` looks at the link itself, while `metadata` follows it, which is why the symlink ban works.
`let ... else` lets a lookup stop early with a diagnostic, which keeps the happy path flat.

# Rust lesson: grimoire-packs T04

A struct field's type is a contract: `pack_version: i64` tells the reader and the compiler what TOML may hold, and a wrong type fails to parse with a message.
`#[serde(deny_unknown_fields)]` turns a misspelt key into an error, so a typo such as `licnce` cannot pass review silently.
`toml::from_str::<toml::Table>` checks syntax alone, which lets the code tell a syntax error (MET-01) from a wrong field value (MET-02).
`Option<T>` with `#[serde(default)]` marks a field that may be absent, and the type makes the caller handle the missing case.

# Rust lesson: grimoire-packs T05

`match` on an enum forces every case to be written out, so adding a launch type later makes the compiler point at each mapping that needs updating.
`matches!((self, schema), (Self::Int, Schema::Int { .. }) | ...)` checks a pair of values in one readable expression, with no extra helper.
`let ... else` returns early when a value is missing, which keeps the main path of a check flat.
`windows(2)` gives each neighbouring pair of a slice, which is the simple way to check that sizes strictly increase.

# Rust lesson: grimoire-packs T06 to T11

`chars().count()` counts characters, while `len()` counts bytes, so a title with an accent is measured as the author sees it.
`strip_prefix` returns an `Option<&str>` that is `Some` only when the prefix matches, which makes the `def <name>(` text check a single expression.
`Iterator::take(200).any(...)` stops drawing candidates as soon as one check fails, so a bad generator cannot run forever.
A `Result` from a TOML parse is mapped to a diagnostic with `match`, so a bad file is reported and never panics.
