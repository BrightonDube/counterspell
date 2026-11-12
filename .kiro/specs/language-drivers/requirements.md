# Requirements: language-drivers

## Purpose

Define the canonical language configuration and stdin/stdout driver contract used by the trusted judge. The launch implementation adds JavaScript on Node.js beside Python, exposes the driver boundary needed by the Efficiency Familiar's measured bench mode, and records Java and C++ as explicitly deferred stretch work.

The Python reference solution remains the only authoritative reference for expected outputs. JavaScript, Java, and C++ fixtures are test fixtures only; they must never become per-language production references or an alternate source of expected answers.

## Scope constraints

- Use the canonical `languages/` directory from the repository skeleton.
- Keep language selection, image, source filename, compile command, run command, resource limits, driver path, and bench-mode arguments in `languages/languages.toml`.
- Python and JavaScript are launch languages and are part of the core implementation.
- Java uses `eclipse-temurin`, never the deprecated `openjdk` image. Its first stack setting is `-Xss16m`; deep-recursion fixtures must test the 8–16 MiB range and retain the smallest passing setting.
- C++ uses the `gcc` image and remains `[stretch]`.
- Java and C++ tasks SHALL NOT be started until the core freeze-safe checkpoint has passed.
- The server never runs user or reference code. The trusted judge owns test inputs, expected outputs, comparison, deadlines, timing, and verdicts.
- Drivers emit answers only. They never emit verdicts, expected values, timings, complexity classifications, or Familiar moods.
- All code execution for implementation tests, reference checks, and MCP-related paths goes through the judge's sandbox runner on the separate rootless sandbox daemon.

## EARS acceptance criteria

### Language registry and configuration

1. THE SYSTEM SHALL provide `languages/languages.toml` with one named configuration for each supported language.
2. THE SYSTEM SHALL include a Python launch configuration with the existing Python image, source filename, driver path, compile command, run command, memory limit, per-test time limit, source-byte limit, and bench-mode arguments.
3. THE SYSTEM SHALL include a JavaScript launch configuration with:
   - a Node.js image (`node:22-slim` as the initial image tag);
   - source filename `solution.js`;
   - the JavaScript driver path;
   - no compile command;
   - run command `node --max-old-space-size=192 driver.js`;
   - a declared memory limit and per-test time limit;
   - a source-byte limit;
   - a bench-mode argument that selects the driver's bench mode.
4. WHERE Java is enabled after the core freeze-safe checkpoint, THE SYSTEM SHALL use an `eclipse-temurin:21-jdk` image, compile `Solution.java` and `Driver.java`, and run with `-Xss16m`, `-Xmx256m`, `-XX:+UseSerialGC`, and the compiled class path.
5. WHERE C++ is enabled after the core freeze-safe checkpoint, THE SYSTEM SHALL use a `gcc:14` image, compile with `g++ -O2 -std=c++20`, and run the resulting binary through its driver.
6. THE SYSTEM SHALL treat image tags as development defaults and SHALL support production image digests without changing the driver contract.
7. Sandbox images SHALL be pre-pulled during deployment. A run SHALL never pull an image from a registry.
8. An unsupported or malformed language configuration SHALL fail before a job is queued, with an actionable configuration error rather than a user wrong-answer verdict.

### Common driver protocol

9. THE SYSTEM SHALL define one versioned JSON-lines protocol shared by Python and JavaScript and followed by later language drivers.
10. For each call, the judge SHALL write one UTF-8 JSON object to the driver's stdin in the form `{ "t": <non-negative integer>, "args": [<JSON value>, ...] }`.
11. The driver SHALL interpret `args` according to the problem signature, invoke the user's function, and write exactly one UTF-8 JSON object followed by a newline in the form `{ "t": <same integer>, "out": <JSON value> }`.
12. THE SYSTEM SHALL preserve the call identifier and JSON value semantics across the input/output boundary. It SHALL not reorder object keys as a semantic operation, coerce booleans to numbers, convert null to an empty value, or silently narrow supported numeric values.
13. Supported driver values SHALL be limited to the JSON-safe signature types in the system design: integers, finite floats, booleans, strings, lists, nested lists, tuples represented by JSON arrays, and null where the problem signature allows it. The pack validator SHALL reject or otherwise exclude values that cannot be represented exactly by a launch language.
14. The driver SHALL use the problem signature to map JSON arguments to the user's function parameters and SHALL validate the return value against the declared JSON-safe return type before emitting it.
15. A driver SHALL emit no stdout other than protocol answer objects. User-level stdout produced while invoking the function SHALL be captured or redirected so it cannot corrupt the protocol stream.
16. Driver diagnostics, traceback text, compile diagnostics, and setup failures MAY go to stderr, subject to the sandbox output limits. They SHALL never be encoded as a verdict or as a fake answer on stdout.
17. If the user's function raises, exits, returns an unsupported value, or produces an invalid protocol response, the driver SHALL not manufacture an answer. The judge SHALL classify the resulting condition from the process outcome and its own limits.
18. If user code prints text resembling `{ "verdict": "accepted" }`, the worker SHALL treat it only as user output to capture or compare. It SHALL never affect the verdict.
19. A driver SHALL load the user's source once per process and SHALL support multiple sequential calls in one process. State persistence between calls SHALL match the documented judge lifecycle and SHALL not expose expected outputs to the user.
20. The protocol SHALL be documented next to the driver templates, including framing, supported values, failure behavior, source export conventions, and the protocol version.

### Python and JavaScript launch drivers

21. THE SYSTEM SHALL provide `languages/drivers/python/` and `languages/drivers/javascript/` templates under the canonical layout.
22. The JavaScript driver SHALL load `solution.js` in the Node.js sandbox and resolve the problem function using the problem signature's function name. The supported export convention SHALL be documented and SHALL work for the starter code generated by the pack system.
23. The Python and JavaScript drivers SHALL have equivalent observable protocol behavior for the same signature, generated arguments, known-correct fixture, and expected result.
24. The JavaScript driver SHALL use Node's JSON parser/serializer and SHALL reject non-finite numbers, `undefined`, symbols, functions, and other values that cannot be represented by the canonical protocol rather than silently changing them.
25. The JavaScript driver SHALL support the launch function signatures used by all current packs, including scalar parameters, strings, booleans, arrays, nested arrays, and JSON-compatible object values where a pack declares them.
26. Drivers SHALL not depend on network access, writable host paths, environment secrets, or language-specific reference outputs.
27. The driver templates SHALL be copied into each sandbox as read-only support files. User source SHALL be the only submitted source file and SHALL remain subject to the server's size, parse, and plain-text validation.

### Efficiency Familiar bench mode

28. WHEN correctness tests for a Run or Submit pass and the problem declares a bench specification, THE JUDGE SHALL be able to restart or switch the driver into bench mode inside the sandbox.
29. Bench mode SHALL be selected through the configured driver argument, initially `--mode bench`, without changing the JSON-lines answer protocol.
30. In bench mode, the driver SHALL load the user's code once, accept repeated calls, and emit only the raw answer objects. It SHALL not measure elapsed time, choose input sizes, fit a complexity class, compare against the reference, or set a Familiar mood.
31. THE JUDGE SHALL generate benchmark inputs from the problem's `bench` specification, send two untimed warm-up calls, then send five timed calls per size and retain the median. The worker's monotonic clock SHALL own the timing.
32. The judge SHALL use the documented budget: up to three benchmark sizes and 1.5 seconds for Run, and five to six sizes and 4 seconds for Submit, stopping early when a point exceeds its cap or the overall budget expires.
33. THE JUDGE SHALL persist raw benchmark samples in `run_timings` and SHALL emit only public `bench_start`, `bench_point`, `complexity`, and `familiar` events through the existing run-event contract. No hidden input or expected output may appear in a public event.
34. The driver layer SHALL expose enough metadata for the Efficiency Familiar implementation to distinguish test mode from bench mode, while the measured complexity classification and mood mapping remain owned by the Familiar/judge design.
35. IF a problem has no bench specification, THE JUDGE SHALL skip bench mode. The Familiar SHALL use the pass/fail path defined by the Efficiency Familiar feature.
36. IF a benchmark call fails, times out, exceeds output or memory limits, or loses its process, THE JUDGE SHALL report the corresponding infrastructure or user-process outcome and SHALL never report a successful complexity measurement.

### Stretch languages

37. Java support SHALL be represented as `[stretch]` tasks and SHALL remain disabled in the launch registry or explicitly marked unavailable until the core freeze-safe gate is approved.
38. Java deep-recursion validation SHALL test `-Xss` candidates from 8 MiB through 16 MiB and SHALL keep the smallest setting that passes the designated deep-recursion fixtures. The starting candidate SHALL be 16 MiB.
39. C++ support SHALL be represented as `[stretch]` tasks and SHALL remain disabled in the launch registry or explicitly marked unavailable until the core freeze-safe gate is approved.
40. Java and C++ drivers SHALL follow the same protocol and verdict boundary as Python and JavaScript. Their compiler or runtime output SHALL not become a per-language reference.

### Reference authority and correctness properties

41. THE SYSTEM SHALL use the Python reference solution as the authoritative source for expected outputs for every problem and every user language.
42. THE SYSTEM SHALL not create, store, or select a production per-language reference solution for JavaScript, Java, or C++.
43. For every published problem and every enabled language, THE SYSTEM SHALL provide a known-correct test fixture used only by driver integration tests.
44. For generated inputs accepted by a problem schema, the driver round-trip property SHALL hold: encoding the generated arguments, invoking the known-correct fixture, decoding the answer, and re-encoding it SHALL preserve the canonical JSON value exactly.
45. For generated inputs, the known-correct fixture for each enabled language SHALL produce an output that matches the Python reference output under the pack's trusted comparator.
46. The correctness suite SHALL run generated-input comparisons for every problem, not only hand-picked samples. It SHALL report the problem slug, language, seed, and generated input when a property fails, subject to not leaking hidden production test data.
47. The suite SHALL verify that the same input and fixture produce the same answer across repeated driver calls, unless the problem explicitly declares nondeterminism, which launch packs SHALL not do.
48. The suite SHALL verify that driver stdout contains only valid answer objects and no verdict, timing, expected-output, or complexity fields.
49. The suite SHALL execute reference and fixture code through the trusted sandbox path. No test command for this feature may execute untrusted or reference code directly on the developer host.
50. The implementation SHALL add property-based and integration coverage without changing the six-package Cargo workspace or creating a second language subsystem.

## Out of scope

- Implementing the Efficiency Familiar's statistical fit, mood mapping, SVG, GSAP animation, speech bubble, or UI.
- Adding Java or C++ before the core freeze-safe checkpoint.
- Adding per-language production references or changing the Python reference authority.
- Allowing arbitrary user scripts, multiple uploaded files, network access, host execution, or keyword blocklists as a sandbox boundary.
- Changing the judge's Postgres queue, lease, run-event, or SSE architecture.
- Adding languages beyond Python, JavaScript, Java, and C++.
