# Design: language-drivers

## Design approach

This is a Design-First, low-level feature. The architecture already fixes the trust boundary and the canonical repository layout. This design makes the language registry, driver protocol, Node.js driver, and measured-bench handoff implementation-ready without adding another crate, queue, reference system, or execution architecture.

The implementation has two layers:

1. `languages/` contains declarative language definitions and driver templates. The templates adapt a problem signature and user function to a stable JSON-lines protocol.
2. `crates/judge/` owns the trusted process lifecycle. It loads the configuration, creates the locked-down sandbox through the existing judge sandbox boundary, sends inputs, measures deadlines and benchmark samples, and decides every verdict outside the driver.

`counterspell-engine` remains pure. It may generate benchmark inputs through the existing problem/engine interfaces, but it does not spawn interpreters or know language-specific process details. The Efficiency Familiar owns the later statistical classification and UI mood presentation; this feature supplies measured samples and the driver mode needed to collect them.

## Scope and non-goals

### Included

- A versioned protocol shared by Python and JavaScript.
- The canonical Python driver contract, including any parity changes needed to make it the baseline.
- A Node.js JavaScript driver using `solution.js` and `driver.js`.
- Python and JavaScript entries in `languages/languages.toml`.
- A generic judge handoff for normal calls and `--mode bench` calls.
- Benchmark warm-up, repeat, timeout, raw timing persistence, and public event integration.
- Per-problem, per-language known-correct fixtures used only for generated-input verification.
- Property-based and integration-test plans for round trips, reference agreement, determinism, and stdout purity.
- Java and C++ designs recorded as `[stretch]` tasks with a freeze-safe gate.

### Deferred

- The Efficiency Familiar's slope/model fit, confidence policy, mood mapping, animation, and AI speech.
- The sandbox implementation itself, which belongs to the judge-sandbox work. This feature calls its existing runner and does not weaken its isolation controls.
- New problem packs or production per-language reference solutions.
- Java and C++ implementation before the core checkpoint.

## Canonical file and module map

```text
languages/
├── languages.toml                         # registry consumed by the trusted judge
├── protocol.md                            # versioned stdin/stdout contract
├── drivers/
│   ├── python/
│   │   └── driver.py                      # Python baseline template
│   └── javascript/
│       └── driver.js                      # Node.js launch template
└── fixtures/                              # optional shared driver fixtures/tests only

packs/<slug>/
└── fixtures/
    ├── python/known_correct.py            # test-only, not a production reference
    ├── javascript/known_correct.js        # test-only, not a production reference
    ├── java/known_correct.java            # [stretch]
    └── cpp/known_correct.cpp              # [stretch]

crates/judge/src/
├── languages.rs                            # TOML registry and validation
├── driver_protocol.rs                      # framing and answer validation
└── bench.rs                                # benchmark process handoff and samples
```

The exact Rust module split may follow the existing judge-sandbox implementation, but responsibilities must remain equivalent. No seventh package, `language-drivers` crate, second judge, or host process runner is allowed.

## Configuration design

`languages/languages.toml` uses one table per language. The fields are intentionally close to the system-design example, with only the fields needed to locate the driver and select bench mode added explicitly:

```toml
[python]
enabled = true
image = "python:3.13-slim"
file = "solution.py"
driver = "drivers/python/driver.py"
compile = []
run = ["python3", "-I", "-S", "driver.py"]
bench_args = ["--mode", "bench"]
mem_mb = 256
time_ms_per_test = 2000
max_source_bytes = 65536
protocol_version = 1

[javascript]
enabled = true
image = "node:22-slim"
file = "solution.js"
driver = "drivers/javascript/driver.js"
compile = []
run = ["node", "--max-old-space-size=192", "driver.js"]
bench_args = ["--mode", "bench"]
mem_mb = 256
time_ms_per_test = 2000
max_source_bytes = 65536
protocol_version = 1

[java]
# [stretch] disabled until the core freeze-safe gate.
enabled = false
image = "eclipse-temurin:21-jdk"
file = "Solution.java"
driver = "drivers/java/Driver.java"
compile = ["javac", "-d", "/box/out", "Solution.java", "Driver.java"]
run = ["java", "-Xss16m", "-Xmx256m", "-XX:+UseSerialGC", "-cp", "/box/out", "Driver"]
bench_args = ["--mode", "bench"]
mem_mb = 512
time_ms_per_test = 2000
max_source_bytes = 65536
protocol_version = 1

[cpp]
# [stretch] disabled until the core freeze-safe gate.
enabled = false
image = "gcc:14"
file = "solution.cpp"
driver = "drivers/cpp/driver.cpp"
compile = ["g++", "-O2", "-std=c++20", "-o", "/box/out/main", "driver.cpp"]
run = ["/box/out/main"]
bench_args = ["--mode", "bench"]
mem_mb = 256
time_ms_per_test = 2000
max_source_bytes = 65536
protocol_version = 1
```

Implementation details:

- The TOML loader rejects missing required fields, duplicate language names, unsupported protocol versions, negative limits, empty run commands, and enabled languages whose driver file is absent.
- `enabled = false` prevents the API and judge from accepting the language. The Java and C++ entries remain visible to pack authors and implementation work but are not launch options until the gate is explicitly changed.
- `driver` is resolved relative to the repository's `languages/` root. The judge copies that template and the user source into the sandbox working directory. It does not allow a submitted source path to override the driver path.
- `bench_args` is appended to the configured run command only for the bench process. The default test process receives no bench argument.
- Image tags are local/development defaults. Production deployment records immutable digests and pre-pulls the enabled images. The registry must not cause a run-time image pull.
- `max_source_bytes` is a byte limit and complements, rather than replaces, server-side UTF-8, control-character, parse, rate-limit, and queue-cap validation.

## Protocol design

### Signature descriptor

Before starting a driver, the judge writes a trusted signature descriptor at the sandbox working path, for example `/box/signature.json`. It contains only public problem signature metadata:

```json
{
  "protocol_version": 1,
  "function": "two_sum",
  "parameters": [
    {"name": "nums", "type": {"kind": "list", "items": {"kind": "int"}}},
    {"name": "target", "type": {"kind": "int"}}
  ],
  "returns": {"kind": "list", "items": {"kind": "int"}}
}
```

It contains no test cases, expected outputs, hidden data, comparator implementation, reference source, or timing limit. The driver uses the descriptor for function resolution and language-level type validation. The trusted worker still owns the authoritative comparator and expected values.

The driver contract is deliberately function-oriented and has no language-specific verdict protocol:

```text
stdin line:  {"t": 0, "args": [[2, 7, 11, 15], 9]}\n
stdout line: {"t": 0, "out": [0, 1]}\n
```

Rules:

- Each input line is one complete UTF-8 JSON object. The judge sends calls sequentially and waits for the matching `t` before sending the next call.
- `t` is a non-negative call identifier. The output must echo it exactly.
- `args` is an ordered JSON array matching the signature's parameters. The worker constructs it from the canonical generated/test input.
- `out` is the JSON-safe return value. The driver does not include an expected value, comparator result, elapsed time, memory, complexity, or verdict.
- A missing line, malformed JSON, mismatched `t`, invalid output JSON, unsupported type, exception, process exit, or timeout yields no accepted answer. The judge classifies the process outcome using its own trusted rules.
- Blank lines are protocol errors. The driver does not silently skip them, because skipping would make call order ambiguous.
- Drivers flush after every answer. The worker's deadline begins before the call is written and ends when the complete answer line is received.
- User-level printing is captured by the driver where the language permits. Any direct output that still corrupts the protocol is treated as a process/protocol failure, never as a verdict supplied by the user.
- JSON object key order is not a semantic value. Canonical comparison is performed on parsed JSON values by the worker's comparator. Numeric values must remain within the exact representable domain declared by the pack validator.

### Python baseline

The Python driver is the behavioral baseline rather than a reference implementation for user answers. It:

1. Reads `/box/signature.json` and verifies protocol version.
2. Loads `/box/solution.py` once without network or host imports.
3. Resolves the named function.
4. Reads one request at a time, validates the `args` shape, invokes the function, validates the JSON-safe result, and emits one answer object.
5. Captures language-level `stdout` around the invocation so `print()` cannot corrupt the answer stream. Diagnostic failures go to stderr.
6. Supports the same default mode and `--mode bench` mode as JavaScript.

The Python driver must not load or consult the Python reference solution. The judge invokes the reference separately through its trusted sandbox path when a comparison is needed.

### JavaScript driver

The JavaScript driver is `languages/drivers/javascript/driver.js` and runs with Node's standard JSON implementation.

- It loads `/box/signature.json`, checks the protocol version, and resolves the configured function name.
- The supported starter-code convention is CommonJS: `module.exports = function ...` or `module.exports = { functionName }`. The resolver first checks a named export and then accepts a callable default module export when the signature name is the declared function. The chosen convention is documented in the protocol and reflected in generated starter code.
- It calls the function with the ordered `args` array. The first implementation is synchronous; a returned Promise is awaited so asynchronous work cannot be mistaken for a completed answer. Network is still unavailable in the sandbox.
- It rejects `undefined`, `NaN`, positive/negative infinity, symbols, functions, cyclic objects, and other values that JSON cannot represent exactly. It emits JSON `null` only when the declared return value is an actual null.
- It intercepts language-level `console`/stdout writes during the call so the protocol stream remains answer-only. Direct native writes that bypass this interception are unsupported and result in a protocol failure if they corrupt framing.
- It serializes exactly one `{ "t": ..., "out": ... }` object and flushes it before reading the next call.
- It writes exceptions and setup diagnostics to stderr only. It never emits `{ "verdict": ... }`, timing fields, or expected outputs.

### Bench mode

Bench mode changes process purpose, not the answer protocol. The judge:

1. Runs correctness calls with the configured normal command.
2. After correctness succeeds, keeps the same sandbox container and terminates the test driver cleanly.
3. Starts the same driver template with `bench_args`, initially `--mode bench`, and the same signature descriptor and source.
4. Sends two untimed warm-up calls.
5. Sends five calls per generated size while measuring each call from the trusted worker. The driver only answers.
6. Persists raw samples and emits public events through the existing event writer.
7. Stops the process/container when the per-call, benchmark, or total wall-time budget is reached, then performs the existing cleanup.

The driver may use bench mode to avoid test-only setup or to select a persistent, repeatable invocation loop, but it must not alter return-value semantics. The worker, not the driver, owns all timing and classification facts.

For a timed call, the worker validates the returned answer with the trusted comparator when the bench design requires correctness confirmation. Expected outputs remain outside the driver. A benchmark failure cannot produce an Accepted or high-confidence complexity result.

## Judge integration design

The language layer integrates with the existing trusted judge and sandbox runner as follows:

- `languages.rs` loads and validates `languages.toml`, exposes an immutable `LanguageDefinition`, and builds normal or bench argv vectors. It does not create containers itself.
- `driver_protocol.rs` performs bounded line framing, parses the `t`/`out` envelope, enforces the protocol version and JSON-safe value rules, and returns structured process errors. It does not decide Accepted/Wrong Answer.
- The existing sandbox runner receives the image, source, read-only driver template, signature descriptor, resource limits, no-network setting, non-root identity, dropped capabilities, seccomp/default security profile, output caps, and cleanup labels. The server never receives this dependency or socket.
- `bench.rs` coordinates warm-up calls, generated sizes, worker-clock timing, medians, hard caps, `run_timings` inserts, and public `bench_start`/`bench_point` events. It delegates complexity fitting and mood mapping to the Familiar implementation or its agreed judge interface.
- The normal test path sends sample or hidden inputs one at a time. The worker compares `out` to the stored expected value and emits the existing public test event. Hidden expected values never enter the driver or public event payload.
- Ghost hunting uses the same driver protocol and sandbox runner. The engine supplies generated inputs, while the trusted judge compares user and Python-reference outcomes. No driver changes the ghost predicate.
- Compilation failures for stretch languages are process failures classified by the judge. They are never represented as a user-produced verdict object.
- The judge preserves the Postgres queue, lease, run-event, SSE replay, terminal-write, and retry architecture. This feature adds no Valkey queue or event bus.

### Bench event and persistence boundary

The judge writes:

- `bench_start`: public sizes only;
- `bench_point`: public size and median nanoseconds only;
- `complexity`: the later measured classification, confidence, reference class, and safe baseline ratio;
- `familiar`: the later measured mood and reason.

The worker stores raw `n`, `repeat`, and `ns` in `run_timings`. It does not put source code, hidden inputs, expected values, stderr, or raw answer values in public events. The existing ownership check on the run endpoint protects all persisted run data.

## Fixtures and property-based verification

### Fixture location and authority

Each published pack supplies a test-only known-correct fixture for every enabled language under `packs/<slug>/fixtures/<lang>/known_correct.<ext>`. The fixture is loaded only by the language-driver correctness harness. It is not inserted into `reference_solutions`, is not sent to users, and is never selected by the judge as expected-output authority.

The Python pack reference remains the sole expected-output authority:

```text
Python reference → trusted expected value → comparator
JavaScript fixture ──────────────────────┘ (agreement assertion only)
```

The same arrangement applies to Java and C++ once their stretch tasks are enabled.

### Properties

The correctness suite should use `proptest` for Rust-side value generation and the existing pack generators where the property needs pack-specific constraints. It should verify:

1. **Protocol round trip:** generated signature arguments pass through the driver and the known-correct fixture; decoding and re-encoding the answer preserves the canonical JSON value exactly.
2. **Reference agreement:** for every problem, language, and generated input, the known-correct fixture's answer matches the Python reference under the pack comparator.
3. **Repeatability:** the same fixture, input, and driver process produce the same answer across repeated calls.
4. **Frame purity:** each stdout line parses as exactly an answer envelope with the matching `t`; no verdict, timing, expected-output, or complexity field is emitted.
5. **Protocol failure safety:** malformed input, wrong call ID, invalid JSON result, unsupported return value, exception, and timeout do not become Accepted and do not let user output control the verdict.
6. **Bench separation:** bench mode produces the same answer envelope as normal mode for the same call, while only the worker records timing.
7. **Configuration validity:** enabled entries reference existing drivers and valid commands; disabled stretch entries are not exposed as launch languages.
8. **Budget termination:** the judge stops benchmark calls within the configured per-call and total budgets, including when the driver loops or stops responding.

Property failures include the pack slug, language, protocol version, deterministic seed, and generated input. The harness redacts or avoids production hidden data. Code execution in these properties uses the same trusted sandbox runner rather than a local interpreter or compiler.

## Java stretch design

Java is deliberately not part of the launch wave. After the freeze-safe gate:

- Add `languages/drivers/java/Driver.java` using the same JSON-lines envelope and signature descriptor.
- Compile the user's `Solution.java` with the driver using `javac -d /box/out`.
- Start with `-Xss16m`, then run deep-recursion fixtures at candidate values from 8 MiB through 16 MiB. Select the smallest setting that passes all designated fixtures and record that decision in the language config and test evidence.
- Use `eclipse-temurin:21-jdk`, never `openjdk`.
- Validate Java's numeric and JSON conversion rules against the same generated-input property suite. A Java fixture is agreement evidence only, never a production reference.

## C++ stretch design

C++ is deliberately not part of the launch wave. After the freeze-safe gate:

- Add `languages/drivers/cpp/driver.cpp` using the same JSON-lines envelope and signature descriptor.
- Compile with the configured `gcc:14` image, `g++ -O2 -std=c++20`, and a temporary output path under `/box/out`.
- Keep the worker's comparator and deadline authority outside the binary. The executable prints an answer envelope only.
- Validate supported JSON types and generated-input agreement with the Python reference before enabling the language in the API.
- Do not add a C++ reference solution registry or permit host compilation.

## Security and architecture invariants

- The server does not depend on the language drivers and does not access any Docker socket.
- The judge is the only component that starts drivers, and it may do so only through the separate rootless sandbox daemon.
- No driver receives expected outputs, hidden test cases, comparator configuration that could reveal answers, LLM secrets, database credentials, or the main Docker socket.
- The driver cannot declare a verdict, timing, complexity, or Familiar mood by printing JSON.
- The default seccomp profile, no network, read-only root filesystem, bounded scratch space, non-root user, dropped capabilities, no-new-privileges, CPU/memory/PID/wall-time limits, output caps, and cleanup/reaper remain the judge-sandbox contract.
- User stdout, stderr, and answer values are untrusted data. Any UI later rendering them must use plain text.
- No keyword or API blocklist is introduced. The sandbox is the execution boundary.
- The Python reference remains authoritative for all languages. Test fixtures cannot alter production expected outputs.
- Production Rust code follows the workspace Rust rules: no `unwrap()` or `expect()` outside tests, useful contextual errors, and a short Rust lesson with implementation changes.

## Verification plan

Implementation should be verified in this order:

1. Parse and validate `languages/languages.toml` without starting a sandbox.
2. Run protocol unit tests for envelopes, call IDs, JSON-safe values, malformed input, and output purity.
3. Run Python and JavaScript fixture properties through the trusted sandbox path for every available pack.
4. Run a known failing and known passing JavaScript problem through normal judge calls, confirming worker-owned comparison and no driver verdict fields.
5. Run JavaScript bench mode on a problem with a bench spec, confirming two warm-ups, five repeats, worker-owned timing, `run_timings`, and public bench events.
6. Verify timeout, output-limit, runtime-error, and invalid-output paths remain non-Accepted and clean up containers.
7. Run the existing workspace formatting, lint, test, and license checks relevant to changed Rust/config files.
8. Before Java/C++ tasks, record that the core freeze-safe gate passed. After each stretch language, run its full property suite and only then enable its registry entry.

## Open questions

1. **Protocol input envelope.** The system design says JSON lines and typed arguments but does not name the request field. **Assumed choice:** use `{ "t": number, "args": [...] }`, because ordered arguments map directly to the signature and keep the worker's canonical input separate from the driver protocol.
2. **JavaScript module convention.** The system design does not specify ESM versus CommonJS. **Assumed choice:** use CommonJS for launch starters (`module.exports`), with a named-export and callable-module fallback in the driver, because Node's plain `solution.js` then works without a package manifest or import-mode ambiguity.
3. **Signature transport.** The design does not specify how a driver receives the function name and types. **Assumed choice:** the judge writes a non-secret `/box/signature.json` descriptor; expected outputs and test cases are never included in it.
4. **Bench process lifecycle.** “The same sandbox container switches the driver into bench mode” does not require the same OS process. **Assumed choice:** stop the correctness driver and start a clean bench driver in the same sandbox container, preserving image and filesystem isolation while avoiding state leaked from correctness calls.
5. **Bench command configuration.** The illustrative registry has only `run`. **Assumed choice:** add `bench_args = ["--mode", "bench"]` and append it to the normal command, so all languages share one explicit mode contract without duplicating run commands.
6. **JSON numeric domain.** JavaScript and JSON cannot preserve every language integer or non-finite float exactly. **Assumed choice:** launch pack schemas restrict generated values and outputs to a JSON-safe exact domain; the pack validator rejects unsupported numeric declarations rather than silently coercing them.
7. **Known-correct fixture location.** The brief requires fixtures but does not name a path. **Assumed choice:** keep them beside each pack under `packs/<slug>/fixtures/<lang>/`, mark them test-only, and keep the production reference registry Python-only.
8. **Stretch-language gating.** The repository needs the Java and C++ plan visible without exposing unfinished launch options. **Assumed choice:** include disabled registry entries and `[stretch]` tasks, with an explicit core-freeze gate task before any implementation or enablement.
9. **Bench classifier ownership.** Bench collection and Familiar classification are adjacent but distinct responsibilities. **Assumed choice:** this feature owns driver mode, worker timing, raw samples, and events; the Efficiency Familiar feature owns fitting, confidence, mood mapping, and UI wording.
