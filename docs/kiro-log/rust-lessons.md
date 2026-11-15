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

# Rust lesson: grimoire-packs T12

`&Path` is a borrowed view of a path, and `PathBuf` is an owned one, so `validate_static(root: &Path)` reads the caller's path without taking ownership of it.
`Option` combined with `and_then` and `let ... else` lets the pipeline stop the metadata stages in one place when `meta.toml` fails.
The report records every stage it ran or skipped, so a caller can tell a clean pack from a pack that was never fully checked.

# Rust lesson: grimoire-packs T13

`std::process::ExitCode` lets `main` return 0, 1 or 2, while helper functions return a `Result` or a code and never call `exit` themselves.
`<[T; 1]>::try_from(vec)` checks that exactly one positional argument was given, without indexing a vector that might be empty.
A `#[allow(clippy::...)]` with a `reason` is the honest way to silence one lint, because the reason stays visible to the reviewer.
`Result` and `?` keep I/O failures out of the validation report, so exit 2 means the tool could not run, not that a pack is bad.

# Rust lesson: grimoire-packs T14

A function that returns `Result<ProblemRecord, RecordError>` makes "only valid packs" part of the type, so the importer cannot forget the check.
`ok_or_else` turns an `Option` into a `Result` and only builds the error value when it is needed.
`#[derive(PartialEq)]` on a record only works when every field type also implements it, so the derive on `ProblemRecord` pulled `Eq` onto the models it contains.
`Result` distinguishes an unreadable directory from an invalid pack, which keeps infrastructure failures out of the pack-error path.

# Rust lesson: grimoire-packs T16

`#[cfg(test)]` compiles code only for `cargo test`, so test helpers never reach the shipped binary.
A test that reads `src/` as text is a cheap architecture guard: it fails the build when a forbidden call such as `Command::new` appears.
`str::contains` on a list of names is enough for this tripwire, and the self-tests prove the scanner would catch a planted call.
Narrowing a forbidden name (`std::process::Command` rather than all of `std::process`) keeps legitimate uses such as `ExitCode` allowed.

# Rust lesson: grimoire-packs T17

A trait is a promise: any type that implements `PackRunner` must provide `run` and `bench`, so the judge can pass its own sandbox type in.
`&mut dyn PackRunner` takes any implementation at run time, without making the adapter generic over every runner type.
`map_err` converts one error type into another, so a runner fault becomes the engine's `OracleError` and never a program result.
The fake runner in the test shows the rule: a crash is data, but an infrastructure fault is an error the caller must handle.

# Rust lesson: grimoire-packs T18

`&mut dyn PackRunner` lets the caller pick the runner at run time, and each call borrows it only for that call, so the checks never keep the runner.
The sandbox checks keep hidden data in memory only. The reference runs inside the sandbox, so the validator never holds a hidden expected value; it checks only the shape of each result.
A closure passed to the engine's `shrink` can capture `&mut` state. The error from a failed probe goes into an `Option` the closure fills, because `shrink` takes a probe that cannot return an error.
`Result<Report, SandboxFault>` keeps a runner fault separate from a pack error, so a broken sandbox can never be reported as a broken pack.

# Rust lesson: grimoire-packs T19

A runner fault is a separate type (`SandboxFault::Runner`), so the code that reads a fault can never mistake it for a pack error.
A `match` on the enum must name every variant, so adding a new fault kind later forces the diagnostic to be updated.
A declared `[bench]` produces a warning without calling the runner. The test gives the runner a `bench` that always fails, so an accidental call would surface as an error.
Exit code 2 for a runner fault is part of the CLI, which has no runner yet. The library gives the diagnostic now, and the exit path is wired when the judge supplies a runner (T20).

# Rust lesson: grimoire-packs T24

`fs::read_dir` returns an iterator of `Result` values, because each directory entry can fail on its own (for example a race with a delete).
The test handles each entry with `expect` and a message, which is allowed in test code only.
A filter on the file name (`starts_with('_')`) keeps `_test` out of the starter list, and `is_dir` removes stray files such as the README.

# Rust lesson: grimoire-packs T25

A plain `fn` item has its own unique type, so a `vec!` of different model functions needs each one cast with `as Model` to a single fn-pointer type.
A non-capturing closure also coerces to a fn pointer, which is why `diagonal_sum` accepts both `|cell| cell` and `i64::abs`.
The starter vectors are hand traces written as literals. The model functions are only tested against those literals, never used to produce them.

# Rust lesson: grimoire-packs T15

`Schema::check_definition` returns a `Result`, so the generator stage can stop on an invalid schema with an early `return` before the engine runs.
Some engine functions panic on bad input (`clamp` panics when min is above max). The validator must check the input first, because a panic in a check would crash the CLI, not produce a diagnostic.
`#[allow(clippy::too_many_lines, reason = "...")]` silences one lint on one function and records why, so the reviewer can judge the exception.
A test that collects every mismatch before it asserts shows all wrong expectations in one run, instead of stopping at the first.

# Rust lesson: grimoire-packs T19 (exit path, supersedes the earlier T19 note)

`Option<&mut dyn PackRunner>` lets one function serve production (no runner, so it prints SBX-07) and tests (a fake runner) without a second code path.
`&mut *runner` reborrows the runner for one call, so the loop can use it again for the next pack. Passing `runner` directly would move it on the first pass.
`collect()` into `Result<Vec<_>, _>` stops at the first `Err`, so the first runner fault ends the stage without extra control flow.
A `#[cfg(test)] mod tests` inside `main.rs` can call private functions through `use super::*`, so the exit path is tested without starting a process.

# Rust lesson: run-lifecycle T01

A crate's `[dependencies]` are what ships; `[dev-dependencies]` only compile for tests. Keeping `sqlx`, `axum`, `tokio` and `bollard` out of `counterspell-common` means the shared types compile anywhere without a database or Docker.
Feature flags such as `features = ["v4", "serde"]` switch on optional code in a dependency. Enabling only what is needed keeps the build small and the licence surface clear.
`#![warn(missing_docs)]` at the top of `lib.rs` makes the compiler ask for a doc comment on every public item.

# Rust lesson: run-lifecycle T02 and T03 (partial)

A `macro_rules!` block can write the enum, its text form and its list once, so `as_str`, `parse` and the serde names cannot drift apart. The `match` in `as_str` has no wildcard arm, so adding a variant without a text form is a compile error.
`matches!` tests several variants in one readable expression. The transition table is written this way so each row reads like the design table.
Put `use` lines and helper functions above a `#[cfg(test)] mod tests`. Clippy's `items_after_test_module` lint flags items that come after the test module.

# Rust lesson: run-lifecycle T05-T07 (no Rust changed)

The SQL schema is the contract that the Rust code relies on. Its CHECK constraints reject bad rows even when a bug skips the Rust checks.
A trigger runs for every update, so a rule written there applies to every writer, including ones that bypass the Rust code.
Role grants list the exact columns each writer may change. That keeps the server unable to write results, even if it is compromised.

# Rust lesson: run-lifecycle T02

A test that only checks behaviour that already exists still earns its place when it can fail. Mutating one message to a duplicate made the distinctness test fail, which shows the table is protected.
`HashSet` collects values and drops duplicates, so comparing its length with the list length is a short, exact duplicate check.
Putting `serde_json::from_str` in a test shows that unknown text is an `Err`, so the wire format rejects values the code does not know.

# Rust lesson: run-lifecycle T04

A `enum` with data, such as `PublicTest::Hidden`, cannot hold an input field, because the variant has no such field. That makes a leak a compile error instead of a test failure.
Private fields plus a constructor (`SampleDetail::new`) let the type cut values to a size before they leave. Callers cannot skip the cut.
`match` on an enum with no `_` arm means a new event kind must have a name and a payload before the code compiles.
Serialising to `serde_json::Value` lets a test check the exact keys a browser will see.

# Rust lesson: run-lifecycle T03

`status_after_event` returns `Option<RunStatus>`, so "not allowed" is a value the caller must handle. A `bool` plus a separate "what is the new status" call would let the two drift apart.
`bool::then_some` turns "is this move allowed" into an `Option` in one line, and the table in `can_transition` stays the only copy of the rules.
`proptest` runs a property against hundreds of random inputs and shrinks a failing case to a small one. The walk test uses it for event sequences.

# Rust lesson: run-lifecycle T08

A package with both `src/lib.rs` and `src/main.rs` builds two targets. Tests import the library, and the binary stays a thin shell.
Cargo features are switches. `testkit` compiles helper code only when asked, so a release build cannot contain it.
`Result<T, ConfigError>` returns a named reason for each bad setting. The message names the variable and never prints its value.
A `pub(crate)` constructor is a compile-time guard: code outside the crate cannot forge a claim token. Unused items fail Clippy with `-D warnings`, so an item waits until its first caller.

# Rust lesson: run-lifecycle T09

`#[sqlx::test]` gives each test its own database, so the tests can run in parallel. Cluster-wide objects such as roles are shared, which is how a parallel race showed up.
`Result<T, sqlx::Error>` lets a test check the exact database error code (for example `23514` for a CHECK violation) instead of "some error happened".
A test that expects a failure must still fail loudly when the statement succeeds. The helper panics in that case.
`RunStatus::ALL` plus two nested loops checks all 81 pairs, so the Rust and SQL rules are compared exhaustively, not by sample.

# Rust lesson: run-lifecycle T10

A `Transaction` rolls back when it is dropped without `commit()`. The role tests end with `rollback()` on purpose, so nothing they write survives.
`SAVEPOINT` is a sub-transaction. After a denied statement, `ROLLBACK TO SAVEPOINT` clears the error and keeps the outer transaction usable.
`&mut **tx` and `&mut *tx` pass the transaction's connection to a query. Both forms appear in these tests; the difference is one extra dereference.

# Rust lesson: run-lifecycle T12

A function that takes `&mut Transaction` runs inside the caller's transaction. `append_event_locked` relies on this: the caller decides when to commit, so the fence, the status change and the insert either all commit or all roll back.
`Option::ok_or(...)?` turns "no row" into a typed error that the caller matches on, such as `LeaseLost`. Expected outcomes stay in `Result`, and a panic is kept for real bugs.
`pub(crate)` lets the claim code and the test kit build a `ClaimToken`, while other crates cannot. A `#[cfg(feature = "testkit")]` gate removes code that only tests use from release builds.
A crate can list itself under `[dev-dependencies]` with a feature turned on. Tests then get the feature without callers passing `--features`.

# Rust lesson: run-lifecycle T11

`FOR UPDATE SKIP LOCKED` lets many workers ask for work at once. A row locked by one worker is skipped by the others, so nobody waits and nobody gets the same job.
`let ... else` handles the "nothing to claim" case early. The `return` drops the transaction, which rolls back, so no empty write is left behind.
`#[derive(Clone)]` without `Debug` on `ClaimedRun` is a small guard: the source code it holds cannot be printed by a `{:?}` in a log line.

# Rust lesson: run-lifecycle T13

`Option<T>` from `fetch_optional` plus `ok_or(QueueError::LeaseLost)` turns "no matching row" into a named outcome. A stale worker gets a value it can match on, not a crash.
A `struct` with `Copy` (such as `JobSnapshot`) is cheap to pass around when several branches need the same fields.
`saturating_add`, `saturating_mul` and `min` keep retry arithmetic from overflowing, so a bad counter cannot panic the worker.
`#[cfg(test)] mod tests` keeps unit tests next to the pure functions they check, such as backoff and sanitising.

# Rust lesson: run-lifecycle T14

`?` returns early from a function, and the transaction then rolls back when it is dropped. A failure in any step therefore leaves no partial result, which the atomicity test checks by forcing a duplicate row.
Enums with data (`TestRecord::Sample` and `TestRecord::Hidden`) make the hidden-test rule a type rule: the hidden variant has no field that could carry the input.
`u32::try_from(...)` into `i32` returns a `Result`, so an out-of-range count becomes a named error (`InvalidResult`) instead of a silent wrap.

# Rust lesson: run-lifecycle T15

`tokio::select!` waits on two futures and runs the branch that finishes first. The reaper uses it to choose between "the timer ticked" and "shutdown was requested".
`watch::channel` is a single value that many tasks can read. Setting it to `true` is the shutdown signal, and a task that starts late still sees it.
Finding a candidate without a lock, then locking and checking it again, is a common pattern. The second check is what stops two reapers from acting on the same row.

# Rust lesson: run-lifecycle T16

A trait with a generic `E: RunExecutor` parameter is checked at compile time for each executor type. A `dyn` trait object would box the future on each call, so the worker stays generic.
`RunContext` is the only thing an executor receives. It has an `emit` method and a stop check, and nothing else, so the type system blocks a verdict write.
Naming: the design sketch reused `ClaimedRun`, which already exists in the queue module, so the executor's input is `RunInput`. Two types with one name would be hard to read.

# Rust lesson: run-lifecycle T17

`tokio::spawn` runs a task on its own. Aborting the task that spawned it does not stop the child, so a heartbeat needs an explicit stop. `JoinHandle::abort_handle` lets the caller stop a task even after `timeout` has taken the handle.
A `JoinError` from a spawned task is how a panic reaches the caller. The worker turns it into a retry, so one bad run cannot stop the worker.
`Arc<Self>` lets several spawned tasks share the worker without copying it. The `Semaphore` permit moves into the task, so the slot frees itself when the run ends.

# Rust lesson: run-lifecycle T17 (crash fix)

Dropping a future cancels it. The worker now keeps its heartbeat as a future inside the run's own task, so a dropped worker stops renewing its lease.
`AbortOnDrop` is a small guard with a `Drop` impl. When the run's task is dropped, the guard aborts the spawned program too. Drop runs on every exit path, so cleanup cannot be forgotten.
`tokio::select!` with `if !flag` skips a branch once it is done. `std::pin::pin!` pins a future on the stack so it can be polled again in a loop.

# Rust lesson: run-lifecycle T19

`main` returns `Result`, and the `?` operator passes each startup error up to the runtime, so the process exits non-zero. The judge's logic lives in `service::serve`, not in `main`, so a test can run it without a process.
`tokio::select!` over `ctrl_c()` and a Unix `SIGTERM` stream waits for whichever signal comes first. The `#[cfg]` blocks keep the code building on non-Unix targets too.
A function that only returns a value needs no `async`. Clippy flags an `async fn` with no `.await`, so `NoExecutor` returns `std::future::ready(...)`.

# Rust lesson: run-lifecycle T18

`proptest` generates many random call sequences and shrinks a failing one to a short example. Here each sequence is a list of enum values (`Op`), so the generator and the code under test share one type.
A model is a second, simpler copy of the state. The test checks that the database and the model agree after every call, so one wrong prediction stops the case at once.
`TestRunner::run` runs an async test body by calling `block_on` on a current-thread runtime, which lets a synchronous property test drive `sqlx` calls.

# Rust lesson: run-lifecycle T20

`axum` turns any type that implements `IntoResponse` into an HTTP reply. `ApiError` is that type here, so a handler can return `Result<_, ApiError>` and the `?` operator sends the error as the JSON envelope.
A `Display` value passed to `ApiError::internal` is logged on the server and replaced with a fixed message. The client never sees the database or path text.
A library target (`lib.rs`) holds the router and the config. `main.rs` only wires them together, so the integration tests build the same router the binary serves.
`Option::filter` and `let ... else` keep the config reader short: a missing or empty value becomes a named error in one line.

# Rust lesson: run-lifecycle T21

A trait such as `UserResolver` is a seam. Production code stores it as `Arc<dyn UserResolver>` in `AppState`, so tests can swap in a different implementation with `with_user_resolver`. Only object-safe traits can do this, which is why the seams are plain `fn`, not `async fn`.
An axum extractor is a type that implements `FromRequestParts`. Putting `CurrentUser` in a handler's arguments makes the handler reject anonymous callers before its body runs.
Clippy warns about `async fn` with no `.await`. Returning `std::future::ready(result)` from a non-async `fn` keeps the future `Send` and avoids the warning.
`#[cfg(feature = "testkit")]` removes the bearer-token resolver from release builds, so a header can never sign a user in outside tests.

# Rust lesson: run-lifecycle T22

`Result<T, E>` lets the validator return the first problem it finds, and `?` stops the checks early. The cheap checks (size, then characters) run before the parser, so the parser only sees input that is already bounded.
A tree-sitter tree can be deep, so the code that finds the first error walks it with a cursor in a loop, not with recursion. A recursive walk could overflow the stack on hostile input.
`std::str::from_utf8` is the safe way to check text: it returns `Err` on bad bytes and never panics.
Proptest feeds random bytes into the validator. It checks that the code never panics, which matters because this runs on untrusted input.

# Rust lesson: run-lifecycle T23

A trait with `async` methods can be called through generics, but it cannot be stored as a `dyn` object without boxing. So the server stores one concrete `Limiter` enum and matches on it. The enum implements the trait, so callers see only the trait.
The Lua script runs in Valkey as one step, so no other client can slip between the increment and the expiry. A Rust `async` block cannot give that guarantee, so the logic lives in the script.
`Result` is not the right type for a rate-limit outage. The policy is to allow the run and log it, so the limiter returns a `Decision::SkippedFailOpen` variant instead of an error the caller might forget to handle.
`Sha256::digest` hashes the client address before it becomes a key. The raw address never reaches Valkey.

# Rust lesson: run-lifecycle T24

A `sqlx` transaction is rolled back when it is dropped without a commit. The code still calls `rollback()` on each early return, so the intent is visible to a reader.
`&mut *tx` turns a `Transaction` into the `&mut PgConnection` that the helper functions need, and clippy asks for plain `&mut tx` where auto-deref already does the same thing.
`pg_advisory_xact_lock` takes a lock that ends at commit or rollback. It is the database equivalent of a per-user mutex, and no Rust `Mutex` could cover several server instances.
A `match` on an enum returned by a helper (`Replay`) keeps each outcome explicit. A missing case would stop the build.

# Rust lesson: run-lifecycle T25

Axum runs extractors in argument order and the body must come last. Taking the body as `Result<Bytes, BytesRejection>` lets the handler turn a too-large body into the same JSON error as every other failure, instead of a plain-text reply.
The checks run in a fixed order, and each one uses `?` to return early. Because nothing is written before the last check, an early return leaves the database untouched.
`ok_or_else(...)` builds the error only when the value is missing, so a successful check does no work on the error path.
Clippy's `needless_pass_by_value` asks for `&Decision` here, because `enforce` only reads the value. Passing a reference is the cheaper choice for a type that is not `Copy`.

# Rust lesson: run-lifecycle T26

`Option` in a struct maps to a SQL `NULL`. `fetch_optional` returns `None` when no row matches, and `ok_or_else` turns that into the same `404` for a missing, foreign or malformed id, so the caller cannot tell them apart.
`serde_json::from_value` into a struct with only three fields copies only those keys. Unknown keys are ignored by default, so a whitelist is one small struct, not a hand-written filter.
Ownership is checked in the SQL `WHERE` clause, so a query for another user's row returns no row at all. Checking in Rust after the read would leave the data in memory.
Timestamps are formatted in SQL with `to_char`. That avoids adding the sqlx `time` feature for one column.

# Rust lesson: run-lifecycle T27

`UPDATE ... RETURNING` gives back a value from the row it changed, in one statement. The cancel handler gets the run's status from the same query that sets the flag, so no second read can race with the judge.
`COALESCE(column, now())` keeps the first cancel timestamp. A repeated request changes nothing, which makes the endpoint safe to retry.
`pg_notify` is called inside a query and sends only a hint. The payload is the run id, and the judge still reads the flag from Postgres.
A `Result` with `?` on each step means a missing run and a foreign run return the same error value, so the response cannot reveal which one happened.

# Rust lesson: auth T02

`Redacted<T>` is a small wrapper with a hand-written `Debug` that prints `[redacted]`. A struct that derives `Debug` then shows that text for every secret field, so a secret cannot leak through a log line by accident.
`optional` and `required` return `&str` borrowed from the map, so reading settings copies nothing until a value is kept. Owned `String`s are made only where the config stores them.
`Option<T>` means "feature on or off". An OAuth provider is `Some` only when both of its values exist, so a half-set provider cannot be used by mistake.
A `match` on the pair `(app_env, value)` lists every case. Adding a new environment or transport makes the compiler point at each place that must change.

# Rust lesson: auth T03

`EmailAddress::parse` returns `Result<EmailAddress, EmailError>`, so the only way to hold an address is through the checks. Code that receives an `EmailAddress` never needs to check it again.
`trim`, `to_ascii_lowercase` and `split_once` each return a new value or a borrowed slice. Nothing is changed in place, so the rules read from top to bottom.
A hand-written `Debug` prints `[redacted]` for the address. A derived `Debug` would put personal data into any log line that prints the struct.
Proptest checks the rules on thousands of random inputs. A property that fails prints the smallest input that breaks it, which makes the bug easy to read.

# Rust lesson: auth T04

`Option<&str>` is a borrowed, optional value. The function returns an owned `String` because the default and the accepted path must outlive the request body.
A `match` guard (`Some(raw) if ...`) checks the value and binds it in one step. Anything that does not match falls through to the safe default.
`percent_decode_str` returns a `Cow<str>`. The check runs on the decoded text too, so an encoded `%2f%2f` cannot slip past a check on the raw text.

# Rust lesson: auth T05

`RawToken::generate` returns `Result`, because the operating system's random source can fail. The error becomes `AuthError::Internal` instead of a panic, so the caller decides what to do.
`#[derive(Debug)]` is not used on `RawToken`. A hand-written `Debug` keeps the secret text out of logs, and the type does not derive `Clone`, so copies of the secret are deliberate.
`Sha256::digest(...).into()` turns the hash output into a plain `[u8; 32]`. A fixed-size array can be compared, copied and used as a map key.
`proptest!` runs one property over many random inputs. The model (`model_valid`) is written separately from the code under test, so the test cannot pass by copying the implementation.

# Rust lesson: auth T06

An `enum` such as `OAuthDecision` lists every outcome. `match` with no `_` arm makes the compiler flag each place that does not handle a new outcome.
`SignIn(UserId)` can only carry the identity's owner. A function cannot return "sign in to the owner of this email", because no such variant exists.
`#[allow(clippy::...., reason = "...")]` silences one lint on one item and records why. It is preferred over a crate-wide allow.

# Rust lesson: auth T07

This task is SQL only, so there is no Rust lesson. The point is that a `CHECK` constraint or a trigger refuses bad rows for every writer, including a future Rust bug, while application code only protects the path it runs on.

# Rust lesson: auth T09

This task is SQL only, so there is no Rust lesson. The useful idea is `SECURITY DEFINER`: a function runs with its owner's rights, so the server role can call it without being allowed to change the table directly. A fixed `search_path` stops a caller from putting a look-alike function in front of the real one.

# Rust lesson: auth T10

`async fn` returns a future that does nothing until it is awaited. Each database call ends in `.await.map_err(database)?`, so a failure is logged once and turned into one safe error type.
`Arc<dyn Clock>` lets production pass the system clock and tests pass a clock they can move, with no change to the session code.
`Option` from `fetch_optional` becomes `let Some(..) = row else { return Ok(None) };`. The early return keeps the happy path flat.

# Rust lesson: auth T11

An axum extractor is a type that implements `FromRequestParts`. A handler that takes `AuthUser` cannot run unless the extractor succeeds, so the user id never comes from a parameter the client can change.
`UserResolver` became an `async_trait`, because the session check needs the database. Trait objects (`Arc<dyn UserResolver>`) cannot return a plain `async fn`, so `#[async_trait]` boxes the future.
The `CookieSlot` is an `Arc<Mutex<..>>` placed in the request. The extractor writes to it and the middleware reads it after the handler, so a cleared cookie still reaches the browser when the handler returns an error.

# Rust lesson: auth T12

A property test generates many random operation lists and checks a model after each step. When one fails, proptest shrinks the list to the shortest one that still fails, so the printed sequence is the one to read.
The model is a small pure struct with its own rules (`works_at`). It is written separately from the code under test, so a bug in the service cannot match a bug in the model by accident.
`prop_assert!` returns an error instead of panicking. That lets the shrinker keep trying smaller inputs.

# Rust lesson: auth T13

`PgPool` is cheap to clone, and it is a handle to a shared pool. `tokio::spawn` needs `'static` data, so each racing task gets its own clone of the store and of the token string.
Single-use is one `UPDATE ... RETURNING` statement. The database lock on the row decides the winner, so no Rust mutex is needed and no read-then-write race exists.
`.bind(user.map(UserId::as_uuid))` passes the method as a function value. Clippy prefers this to a closure that only calls a method.

# Rust lesson: auth T15

`Hmac::<Sha256>::new_from_slice` and `mac.update` build a keyed hash. A key derived from the session secret means a stored counter key cannot be matched back to an address by someone who only has Valkey.
`match` arms with a guard (`Decision::SkippedFailOpen if action.fails_closed()`) let one rule decide between two outcomes, and order matters: the first matching arm wins.
A small private trait (`CalendarText`) adds one method to `time::Date`, which keeps the formatting next to the code that uses it.

# Rust lesson: auth T16

A middleware is a function that takes the request and a `Next`. It can answer early (the guard returns `403` and never calls `next.run`), or pass the request on and change the response afterwards.
`axum::middleware::from_fn_with_state` passes shared data, here an `Arc<WebOrigin>`, into the middleware without a global.
The body guard reads the bytes and then rebuilds the request with `Request::from_parts`. Reading the bytes is the only reliable way to know whether a body exists, because headers can be missing.
`let ... else` keeps the happy path flat when a match has one failure arm.

# Rust lesson: auth T24

`spawn_blocking` moves CPU-heavy work (Argon2) to a thread pool for blocking code. The async workers stay free to answer other requests while a hash runs.
A `Semaphore` is a counter of permits. `acquire_owned` gives a permit that holds its place until it is dropped, so the gate limits hashing at the same time across all requests.
Axum 0.8 does not accept `Option<ConnectInfo<..>>` as an extractor. A required extractor plus `MockConnectInfo` in tests is the simple fix.

# Rust lesson: auth T26

`Option<T>` means "feature on or off" here. `DemoConfig` is `Some` only when both demo variables are set, so the seed and the config route check one value, not two flags.
`then_some` turns a `bool` into an `Option`: `(verdict != Verdict::Rejected).then_some(stored)` keeps the stored hash only when it still accepts the password.
The seed calls a database function instead of writing `users` itself. The function is the only place that can touch the demo row, so the Rust code cannot bypass the guards from T09.

# Rust lesson: auth T28

`tokio::spawn` starts a task on the async runtime and returns a `JoinHandle`. Calling `abort()` on that handle stops the task, which is how shutdown cancels the daily reset.
`Arc<dyn Clock>` lets the background task share one clock with the rest of the server. `clock.as_ref()` lends a plain `&dyn Clock` for each call, so the `Arc` never has to move into the call.
`OffsetDateTime` and `Duration` are small values that are copied, so `latest_scheduled_reset` returns a new time without borrowing anything.

# Rust lesson: auth T14

`Arc<dyn EmailSender>` lets handlers hold any sender. Start-up picks the real one, and tests put in `MemorySender`. The trait object costs one pointer indirection, which is fine for mail.
`GuardedSender` wraps another sender. It checks the demo address and the daily cap, then calls the inner `send`, so no caller can skip those rules.
`hit_daily_mail_cap(...).map_err(|_| MailError::Unavailable)?` turns one error type into another. The `map_err` keeps the mapping visible.
`Mutex::lock` fails only if another thread panicked while holding the lock. `unwrap_or_else(PoisonError::into_inner)` takes the data anyway, so no `unwrap()` is needed.
The hand-written `Debug` on `OutgoingMail` hides the body, so printing a mail with `{:?}` can never leak a sign-in link.

# Rust lesson: run-lifecycle T27

`JudgeConfig::from_lookup` takes a closure, so a test can pass its own settings without touching the process environment. Each test gets exactly the values it needs.
`tokio::time::timeout` wraps a `JoinHandle` and returns an error if the run does not end in time. A stuck test then fails with a clear message instead of hanging.
`Arc<Worker>` lets one worker be moved into `tokio::spawn` while the test keeps a handle. Shared ownership is explicit, and nothing is copied.

# Rust lesson: run-lifecycle T28

`broadcast::Sender` sends one value to many receivers. Each receiver gets its own copy, and a receiver that falls too far behind gets `RecvError::Lagged` instead of blocking the sender.
`PgListener::try_recv` returns `Ok(None)` when the connection drops. The caller then decides what to do, which is why the waker sends a "read again" marker at that point.
`Result` patterns can name the error (`Err(elapsed)`) or match a whole group (`Ok(Ok(_) | Err(RecvError::Lagged(_)))`). Clippy asks for named bindings when a bare `Err(_)` would hide a failure in a test.

# Rust lesson: judge-sandbox T01

Cargo features switch code on and off at build time. `bollard` is an optional dependency behind the `sandbox` feature, so builds without it never compile the Docker client.
`clippy.toml` `disallowed-methods` turns a forbidden function into a lint error. Judge code cannot call `Command::new` by accident.
Configuration errors are a typed enum (`thiserror`). Each variant names the variable and never echoes its value, so a secret in a bad setting cannot leak into a log.

# Rust lesson: judge-sandbox T04

A newtype with a private field makes an invalid value impossible to build. `PinnedImage` has no public constructor, so the only way to get one is `parse`, which refuses tags. The compiler enforces the rule everywhere the type is used.
`#[serde(deny_unknown_fields)]` makes a typo in the catalogue a load error instead of a silently ignored setting.
`toml::Table` is a sorted map, so validation errors appear in a stable order, and the first broken language is always the same one.

# Rust lesson: judge-sandbox T05

A plain data struct holds the policy, and a separate function checks it. The checker is what catches a weakened value, so it gets its own tests: each test weakens one field and expects exactly one named error back.
`#[derive(thiserror::Error)]` on an enum gives each control its own variant with a message. A violation is a value the caller can match on, not a string to parse.
`#[must_use]` on functions whose result matters stops a caller from calling the checker and ignoring its answer.

# Rust lesson: judge-sandbox T06

A value built from typed parts cannot carry a stray path or command. `FileName` refuses separators, so `SandboxFiles` can only hold the three fixed names. The archive builder takes that type, not a string.
`ExecSpec::compile` returns `Option<Self>`. A language with no compile step gives `None`, so the caller cannot forget that case.
`tar::Builder::append_data` writes each entry from a byte slice. Setting the header size and owner first means the archive matches what the sandbox user expects.

# Rust lesson: judge-sandbox T07

`&[u8]` is a borrowed view of bytes, so `split` on a chunk returns slices without copying. The reader copies a segment into its own buffer only after the size check passes.
`std::mem::take` moves a finished line out of the buffer and leaves an empty one behind. No clone is needed, and the buffer is reused.
Bounded memory comes from checking the length before extending a `Vec`. Checking afterwards would let a flood allocate first.

# Rust lesson: judge-sandbox T08 and T09

`serde_json::Value` holds any JSON. Parsing it first and checking keys afterwards lets the code report the exact broken rule (extra key, wrong id) instead of a generic error.
`let ... else` returns early when a pattern does not match, which keeps the happy path flat.
`String::from_utf8_lossy` turns invalid bytes into U+FFFD. `is_char_boundary` finds a safe cut point, so a multi-byte character is never split.

# Rust lesson: judge-sandbox T10

A struct with a private field can only be built in the module that defines it. `Verified(())` is that proof: only `compare.rs` can make one, so only `judge_answer` can make `TestVerdict::Pass`.
A `compile_fail` doc test proves the rule to the compiler. If someone makes the field public, the doc test fails.
`impl Debug` written by hand can hide a value. `Expected` prints a placeholder, so a hidden expected output cannot leak into a log or an error.

# Rust lesson: judge-sandbox T11

An enum with data (`TestVerdict`) plus a `match` makes the compiler list every case. Adding a new verdict later breaks the build until each `match` handles it.
`filter_map` with an `Option`-returning helper drops the passing cases without needing an `unreachable!()` arm, so the code has no panic path.
`min_by_key` picks the lowest ordinal, so the first failure is deterministic, even when the tests arrive in a different order.

# Rust lesson: judge-sandbox T12

`&[u8]` is raw bytes and `&str` is checked UTF-8. `std::str::from_utf8` turns one into the other and returns `Result`, so invalid input can't become a `str` by accident.
`char::is_control` covers every control character in one call, so one rule catches NUL, ESC, BEL and DEL.
A returned `&str` borrows from the input. The source is checked once and never copied.

# Rust lesson: judge-sandbox T13

Enum variants make impossible states unrepresentable. A `Hidden` record has no field for output, so the type itself cannot carry a hidden test's text.
A match guard (`Some(detail) if run.sample && !self.hidden_phase`) checks state before a value is used, so the redaction rule sits in one place.
`#[serde(tag = "kind")]` writes the variant name into the JSON. A reader can tell the two record kinds apart without guessing.

# Rust lesson: judge-sandbox wave 1 review fixes

`serde_json::Value` silently keeps the last of two equal keys, so a checker that parses to `Value` cannot see the duplicate. A small `Deserialize` visitor that reads only the keys (`IgnoredAny` skips the values) catches it.
`match (iter.next(), iter.next())` checks "exactly one" without collecting. `(Some(only), None)` is true only when nothing else follows.
A manual `impl Debug` that prints a placeholder keeps a type's contents out of `{:?}` output, in tests and in logs.

# Rust lesson: judge-sandbox T14

A trait can declare an `async` method by returning `impl Future + Send`. Generics then pick the backend at compile time, so there is no `dyn` and no boxed future.
`std::future::ready(value)` gives a ready future for a fake that does no waiting, which keeps the fake free of `async` functions that never `.await`.
A `Mutex` guard must not live across an `.await`. The fake takes the lock in short sync blocks and releases it before returning, so the future stays `Send`.

# Rust lesson: judge-sandbox T15

`Drop` cannot `.await`, so the cleanup guard moves the container into a spawned task with `Handle::try_current().spawn(...)`. The normal path still awaits `close()`, so the caller knows the removal finished.
`Option::take()` in `close` and `Drop` means the container is cleaned up once. After `take`, the `Drop` impl sees `None` and does nothing.
A `?` inside `prepare` still closes the container, because `open_session` closes it on every error before it returns. Returning the error is then safe, and no code can leak a container by returning early.

# Rust lesson: judge-sandbox T16

`tokio::select!` with `biased;` checks its branches in the order written. Putting the deadline first means a late answer can't win a tie against the clock.
A `select!` arm that does `return` leaves the function at once, so each abnormal end is one arm and one `return`.
`let Ok(lines) = io.stdout.push(&bytes) else { ... };` (let-else) handles the overflow error in one line and keeps the happy path flat.

# Rust lesson: judge-sandbox T17

A `for` loop with `break` and `return` is the normal way to stop early. Each exit is one line and the cleanup after the loop runs once.
`match` on a small enum (`Step::Ran` or `Step::Stopped`) moves a "stopped" result out of the verdict path, so the verdict code never sees it.
`Option::as_mut()` with `let ... else` avoids `unwrap` when the code needs a value it just checked. Clippy's line limit pushed the per-test call into its own function, which also made the loop easier to read.

# Rust lesson: judge-sandbox T18

A `proptest!` block runs many random cases. Each case builds its own runtime with `Builder::new_current_thread`, because a property test is a normal function, not an `async` one.
A test can prove a cleanup rule by breaking it on purpose. With `Drop` cleanup removed, the cancel, panic and fault tests failed. Restoring the file brought them back.
Casting `usize` to `u32` can silently truncate, so clippy warns. `u32::try_from(...)` or a `u32` counter avoids that, and `u32::try_from(...).unwrap()` is acceptable in a test.

# Rust lesson: judge-sandbox wave 2 review fixes

`timeout_at(deadline, future)` bounds an async step by an absolute `Instant`, so a slow daemon cannot hold a slot past the run cap.
`match lines.as_slice() { [] => ..., [line] if held == 0 => ..., _ => ... }` matches on slice shape. It states "exactly one line and nothing after it" in one place.
When a function can fail after an external side effect, do the cleanup before returning the error, so the caller sees a clean state. A `Drop` guard alone only starts the cleanup.

# Rust lesson: judge-sandbox T19

Validating a socket path is a trust decision, so `validate_socket_path` returns a `Result` with a named error for each refusal (empty, relative, URL, main daemon). A bare `bool` would not say why the judge refused.
The daemon's `info` is turned into a plain `DaemonFacts` struct through `From<&SystemInfo>`. Only that conversion touches bollard types, so the checker itself is ordinary data that tests can build by hand.
Missing facts (for example no `SecurityOptions`) become "not rootless" or "not v2". This fails closed: an unknown answer never passes the check.

# Rust lesson: efficiency-familiar T02

An `enum` with no data is a fixed list of names, and the compiler makes every `match` on it cover all names. `ClassId::rank` returns the ordering number from one `match`, so adding a class without a rank fails to compile.
`Mood` is a newtype that holds a private `u8`. `TryFrom` is the only way to build one, so an out-of-range mood cannot exist anywhere in the program.
`impl FromStr` lets `"n log n".parse::<ClassId>()` reject unknown spellings with a named error, instead of accepting a loose string.

# Rust lesson: efficiency-familiar T03

`match` on the verdict first, then on the bench. Each `return` path gives one mood, so the rule reads as the same table the requirements use.
A `match` on an `i16` gap with range patterns (`3..`) covers every integer, so the compiler checks that no case is missing. A gap is a small number, so `i16::from` avoids an `as` cast that Clippy would flag.
Tests that enumerate every combination (`Verdict::ALL` times modes times bench variants) check invariants directly, without a random generator.

# Rust lesson: efficiency-familiar T04

`#[derive(Serialize, Deserialize)]` with `#[serde(deny_unknown_fields)]` turns an unexpected key into an error. A payload from a newer or hostile writer cannot slip extra data into storage.
`#[serde(skip_serializing_if = "Option::is_none")]` leaves a missing value out of the JSON entirely. A "nothing was fitted" summary then holds only the fields that are known.
A `Result` from `to_bounded_json` makes the size cap part of the type: the only way to get an event object is through the cap check.

# Rust lesson: efficiency-familiar T05

Wording is a function from stored facts to text, so every headline can be tested by listing every combination. Each `match` arm names one reason, so the compiler catches a reason with no sentence.
`str::strip_prefix` removes the Run-only "Quick check: " prefix before a test checks the top label. A plain `starts_with` would miss it.
A test that enumerates the whole input space found a real problem: a pleased result began with the top-mood label. The fix was a different sentence, not a weaker test.

# Rust lesson: efficiency-familiar T06

`validate` returns `Result` and checks each rule with `if ... return Err(...)`, so bad input never reaches the arithmetic. Nothing in the fit can panic on user data.
`windows(2)` gives each consecutive pair of points without indexing, so the "strictly increasing sizes" check reads as one line.
A `Default` impl on `FitConfig` keeps every constant in one struct, so calibration (T12) changes one place.

# Rust lesson: efficiency-familiar T07

Each model is a small closed-form weighted least-squares solve (two normal equations). When a coefficient comes out negative, the code sets it to zero and refits the other one, so the result never has a negative parameter.
`f64` can hold `u64::MAX` only approximately. `size_as_f64` is the one place that converts sizes, with a comment on why the rounding is harmless on a log scale.
Modelling `2^n` in log space (`ln t = α + n·ln 2`) avoids `inf` for large `n`. Keeping an overflow-prone quantity in logs is a common numerical habit.

# Rust lesson: efficiency-familiar T08

`slice::windows(2)` pairs consecutive points with `zip`, which reads like the maths. `top_count` uses `usize::div_ceil`, which avoids a hand-written rounding step.
`sort_by(f64::total_cmp)` sorts floats with a total order, so a stray NaN cannot make the sort panic or give a random result.
A test that puts an expected value next to its hand-derived source (`Sxx = 2·(ln 2)²`) caught my own arithmetic error in the noise test. The literal was wrong, not the code.

# Rust lesson: efficiency-familiar T09

The grade is one function with an early `return` for each "Low" reason. Each check reads as one sentence, and the `Vec` of reasons is built in the same order the requirements list them.
Passing `&[...]` slices into pure functions keeps ownership with the caller. The fit never copies the input except when it drops a point for the stability check.
`bool::then_some` turns "class only at medium or high" into one expression, and `Option` makes "no class" impossible to confuse with a class.

# Rust lesson: efficiency-familiar T10

A test helper that generates data needs its own tests, because a property test is only as good as its inputs. The corpus tests check parameter ranges and a hand-checkable series before any fit runs.
`u64` medians are scaled so the largest sits at 1e18. That keeps every value inside `u64` range, which matters because `as u64` from a too-large `f64` saturates silently.

# Rust lesson: efficiency-familiar T11

`proptest!` runs each property on many random inputs and shrinks a failure to a minimal case. Reading "minimal failing input" tells you which shape breaks the rule.
Mutation check: loosening a constant on purpose made P1 fail. A property that never fails proves nothing, so the check is worth the minute it takes.
`prop_assert!` returns an error inside a property rather than panicking, so proptest can shrink the input before it reports.

# Rust lesson: efficiency-familiar T12

Constants live in one `Default` impl, and the calibration table is an `#[ignore]`d test that prints the numbers. Tuning is then visible in a diff and reproducible, not a hidden edit.
A count that must be zero (wrong-at-High) is asserted by a property, not eyeballed from the table.

# Rust lesson: efficiency-familiar T13

The bench seed is an FNV-1a hash over length-prefixed fields. Writing the length first means `"ab" + "c"` and `"a" + "bc"` hash differently, the classic field-boundary bug.
A pinned constant is checked against an independent calculation (Python, same fields). Pinning a value that the code itself produced proves nothing until a second implementation agrees.
`isqrt` by binary search with `checked_mul` avoids both floating-point rounding and a linear loop that would be far too slow for large `n`.

# Rust lesson: efficiency-familiar T12 (revised)

A calibration can disagree with a sampled property. The 4,000-case table showed zero wrong Medium, but a 200,000-case run and a random property run both found one. Larger samples and the property together were needed.
An `as` conversion inside a test helper still trips Clippy's pedantic lints. A narrow `#[allow]` with a reason is clearer than a wider blanket.

# Rust lesson: efficiency-familiar T14

A geometric mean is `exp(mean(ln ratio))`. Working in logs turns a product of ratios into a sum, so one very large ratio cannot overflow the result.
A `BTreeMap` keyed by size matches the two series by size regardless of input order, and its iteration order is sorted, so the result is deterministic.
`let ... else` returns `None` for a missing size without an `unwrap`, which keeps the "not meaningful" case explicit.

# Rust lesson: efficiency-familiar T17
- `match` on an enum with no `_` arm makes the compiler list every case. When the engine adds a growth class, `to_common_class` stops compiling until someone maps it.
- `let ... else` returns early when a value is `None`, which keeps the happy path flat (the run that did not complete gets no mood).
- `clone()` on an `Option` field lets a test read a value without moving it out of the struct; in production code, move the value once and pass it on.
- Pure functions take everything as arguments and return a value. `assess` does no I/O, so its tests need no database and no clock.

# Rust lesson: efficiency-familiar T18
- A trait such as `LoadSource` is a seam: production reads `/proc/loadavg`, and a test passes a scripted reading. The decision code never knows which one it has.
- `tokio::sync::Semaphore` gives a permit as an `OwnedSemaphorePermit`. Dropping the permit returns it, so a bench cannot hold the gate after an early return or an error.
- `Option` means "unknown" here. `is_none_or` lets an unknown reading pass, and only a known reading above the limit discards a bench.
- `parse::<f64>()` accepts `NaN` and `inf`. Checking `is_finite()` as well as `> 0.0` rejects them.

# Rust lesson: efficiency-familiar T24
- `let ... else` and `?` on a custom error type let a bad bench size become one diagnostic. The check stops that size, not the whole pack.
- `u64::try_from(i64)` returns an error for a negative size instead of wrapping, so a bad pack value cannot become a huge number.
- A function that takes `impl Into<String>` accepts both `&str` and `String` messages, which keeps the diagnostic call sites short.
- Matching a text value to an enum with `_ => None` is the right shape for a declared name that might be unknown. Exhaustive matches are for values the code already holds.

# Rust lesson: judge-sandbox T19
- A `Result` returned by a validation function is the trust boundary. `validate_socket_path` turns a raw string into a `PathBuf` only after the empty, relative, URL and main-daemon checks pass, so the rest of the code never sees an unchecked path.
- A `#[cfg(feature = "sandbox")]` module keeps the Docker client out of builds that do not need it. Only the edge module names `bollard`, and the pure checks compile everywhere.
- A test can guard "does not read the environment" by scanning source text. Filtering out comment lines first avoids false positives from doc comments that mention a variable name.

# Rust lesson: judge-sandbox T20
- Converting between two types with the same field names is easier with serde than with a hand-written mapper. The create request serialises to the same JSON shape the daemon returns, so a test can rebuild the daemon's answer from the request and read it back.
- `Option` fields from the daemon mean "not reported". Using `unwrap_or` with a value that the checker rejects (for example `0` for memory) keeps a missing field a visible violation, not a silent default.
- `let ... else` and `?` on `Option` keep the read-back short. A missing config becomes one typed error.

# Rust lesson: language-drivers T01
- A test can read a document and check its examples against the real parser. The test calls `parse_answer` on each answer example, so the protocol text and the judge code cannot drift apart without a failing test.
- Slicing a `&str` with `find` and `&rest[start..]` walks through a file without copying it. Each slice borrows from the original text, and the function returns owned `String`s only for the examples it keeps.
- `serde_json::from_str::<Value>` is enough to check that an example is valid JSON. Use `Value::get` to tell an answer example (it has `out`) from a call example.

# Rust lesson: language-drivers T05 (partial: bench argv)
- `to_vec()` copies the catalogue's command into an owned `Vec`, and `extend_from_slice` adds the bench arguments to that copy. The catalogue value is never changed, so the normal driver command stays as it was.
- A test that checks two commands side by side (normal and bench) catches a shared-state bug where one command leaks into the other.

# Rust lesson: auth T01
- `default-features = false` turns off the crate's optional defaults, so only the features named in `Cargo.toml` are compiled. The lock file then lists what really ships, and we can check it with `cargo tree`.
- A Cargo feature is an opt-in switch, not a version. `rustls-tls` picks a TLS library, which is why it brings in a certificate list with its own licence.
- `cargo deny` reads the licence of every crate in the lock file. One exception can name one crate, so a licence decision never widens to others.

# Rust lesson: auth T14
- `async fn` in a trait needs the `async_trait` macro here, so `Arc<dyn EmailSender>` can hold any sender. Handlers hold the trait object and tests swap in a recorder.
- `reqwest::Error::is_connect()` and `status()` decide whether a retry makes sense. A rejected request (4xx) is not retried, because the same request would be rejected again.
- `error_for_status()` turns an HTTP 4xx or 5xx into an `Err`, so the `?` operator and `match` handle network and HTTP failures in one place.

# Rust lesson: auth T17
- `Option<T>` in a struct field (`mail: Option<MailPort>`) makes "not wired yet" a value. The handler turns `None` into an error with `?`, so a route can never send mail through an unset sender.
- `tokio::spawn` takes ownership. The handler moves the sender `Arc` and the message into the task and returns `202` at once, so the response does not wait for the mail provider.
- `tokio::join!` runs two futures on one task. The concurrency test uses it to race two verifies of one token, and the database's single `UPDATE ... RETURNING` decides the winner.

# Rust lesson: auth T18
- `async_trait` lets an `OAuthProvider` trait object be stored and called with `.await`. Each provider adapter can then sit behind one type.
- Deleting a row with `DELETE ... RETURNING` reads and removes it in one statement. A flow cannot be read twice, even when two callbacks race.
- A type alias such as `FlowRow` names a long tuple. Clippy's complexity lint asks for this, and the alias keeps the column order in one place.

# Rust lesson: auth T19 and T20
- A builder-style API such as `BasicClient::new(...).set_...(...)` returns a new value each call. The final value is used once for the request, and the chain is easy to read with one call per line.
- `serde` skips unknown JSON fields by default, so a provider adding a field does not break sign-in. A required field such as `sub` that is missing becomes a clear error.
- `Option<T>` with `.is_some_and(...)` tests a length limit without unwrapping, which keeps the body cap in one readable line.

# Rust lesson: auth T21
- A handler that must always redirect returns `Response`, not `Result`. Its inner function returns `Result<String, Refusal>`, and the outer function turns each refusal into a redirect code in one `match`.
- `Option` with `ok_or(...)` and `?` makes each callback step stop at the first failure, so the steps run in the order the security rules require.
- `Redirect::to` builds a `303 See Other` response, which makes the browser follow the redirect with GET.

# Rust lesson: auth T22
- A small struct can carry context that a plain error cannot. `Refused` holds the flow purpose next to the reason, so the one place that builds the redirect knows whether to send the browser to the settings page or the sign-in page.
- `match` on the enum from a pure decision function (`LinkDecision`) with no `_` arm. Adding a new decision then stops the build until every caller handles it.
- A unique-constraint error from a race is handled by reading the database again, not by guessing. The `Err(error) if is_unique_violation(&error)` match arm keeps other errors on their own path.

# Rust lesson: auth T23
- `Option::ok_or` turns a missing session into an error, and `Option` comparison (`bound_user != Some(id)`) checks ownership in one line.
- A `match` on `sqlx::Result<PgQueryResult>` uses a match guard (`Ok(result) if result.rows_affected() == 1`). A guard lets one arm mean "the row was updated" and a later arm mean "it was already set", without a nested `if`.
- A shared helper such as `send_in_background` moves an `Arc<dyn EmailSender>` into `tokio::spawn`. The `Arc` is cloned first, so the caller keeps its own handle.

# Rust lesson: auth T25
- Keeping a password hash inside a token's JSON payload means nothing is stored until the mailbox is proven. The payload is read only after the token is consumed.
- `if let Some((id, false)) = account` matches a tuple and a literal in one pattern. Only an ordinary account gets mail, and the same `202` is returned either way.
- The unique email constraint is the final guard. A racing registration gets a database error, and `is_unique_violation` maps it to `account_exists`, so no check-then-insert race exists.

# Rust lesson: auth T27
- A `pub const` list of route paths next to the router gives one place to check. A test sends a request to each path and fails with a clear message if a route is missing, so a route added in one place and not the other is caught.
- A `snapshot` function returns a tuple of counts. Comparing two tuples with `assert_eq!` checks that a restricted request changed nothing, and a failure prints both values.
- A database trigger guards the demo row even when a test uses the superuser pool. The test confirms that the trigger, not the application, refuses the change.

# Rust lesson: auth T33
- `Option<(A, B)>` from `zip` pulls two optional values together, so the router either gets both the auth state and its origin, or neither. A half-configured CSRF guard cannot exist.
- `main` builds everything that can fail (config, state, listener) before the server starts, and each error is turned into an `io::Error` with a message that names the setting and never its value.
- Taking `AppState` by value and moving it into `with_state` at the end avoids cloning it for each route group. Clippy's `needless_pass_by_value` lint flagged the copy I first wrote.

# Rust lesson: auth T08
- A foreign key is a rule the database checks on every write, so it holds even when a test or a second service writes the table. Adding it exposed every test that invented a random user id, which is the kind of gap a lint cannot find.
- Test fixtures build a handle from the full 32-character UUID. A shortened prefix made different ids collide, and the unique index refused the second row.
- `ON CONFLICT (id) DO NOTHING` makes a fixture safe to call twice for the same id, so tests can share one helper.

# Rust lesson: auth T34
- `AppState::with_auth` swaps the user resolver for the session one. A test that asks the resolver directly, with a valid cookie, a forged one and no cookie, shows the production path is not the fixed test identity.
- A helper that reads a cookie from a `Set-Cookie` header uses `strip_prefix` and `find_map`. These return `Option` and avoid index arithmetic on strings.

# Rust lesson: auth T35
- A global `tracing` subscriber must be set once per test binary. The test binary holds a single test for this reason, so no other test can install a second subscriber.
- A `Write` implementation whose clones share an `Arc<Mutex<Vec<u8>>>` lets a `MakeWriter` closure hand the same buffer to every event, including events from blocking threads.
- A sentinel event in the test proves the capture works, so a check that finds no secret cannot pass by seeing nothing.

# Rust lesson: efficiency-familiar T16
- `#[sqlx::test(migrations = ...)]` gives each test its own fresh database with every migration applied, so tests cannot leak rows into each other.
- Each test seeds data as the owner, then switches to a role with `SET LOCAL ROLE` inside a transaction to prove what that role may do.
- A denied statement is wrapped in a `SAVEPOINT` and rolled back, so one expected error does not abort the rest of the test.
- Error codes come from the database (`23514` for a CHECK violation, `42501` for missing privilege), and matching on them keeps the test honest about why a write failed.

# Rust lesson: efficiency-familiar T19
- An `Option<Box<T>>` field keeps a large optional struct from making every enum variant that holds it large. Clippy's `large_enum_variant` lint is the signal to box it.
- `Option::as_deref()` turns `Option<Box<T>>` into `Option<&T>` without moving or cloning the value.
- `Option::and_then` and `map` chain optional values with no `if let` nesting, and `map(ClassId::id)` is preferred over a closure that only calls the method.
- `?` on a `Result` returns the error to the caller, so a serialisation failure becomes one mapped error with no `unwrap`.

# Rust lesson: efficiency-familiar T20
- A trait method that returns `impl Future + Send` lets a generic function await the collector without boxing the future, so the bench path has no `dyn` cost.
- `&mut C` in a generic argument lets a test count how many times the collector ran, because the test keeps ownership of the double.
- Borrowed data in a `match` (`BenchState::Points(&points)`) lives as long as the outcome it borrows, so no clone is needed.
- A pure builder function (`assess_input`) keeps the struct update in one place, and `..` copies the fields that do not change.

# Rust lesson: efficiency-familiar T21
- `sqlx::query(...).bind(...)` takes the values separately from the SQL text, so user-supplied strings never become SQL.
- `rows_affected()` tells you how many rows an `INSERT ... ON CONFLICT DO NOTHING` really added, which is how the store reports "already there".
- `try_from` turns a Rust `u64` into a database `int` with an error, and `?` returns that error, so an oversized value cannot be stored silently.
- A `thiserror` enum with `#[from] sqlx::Error` lets `?` convert database errors into the module's own error type.
