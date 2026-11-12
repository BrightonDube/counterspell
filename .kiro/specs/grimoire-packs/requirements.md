# Requirements: grimoire-packs

## Purpose

Define the reviewed file format for Grimoire problem packs under `packs/<slug>/`, a validator (`counterspell-grimoire validate`) that rejects broken packs before they are imported, and the first original starter packs toward the 10 launch problems.

Sources: `docs/system-design.md` §4 (problem columns), §6 (languages), §8 (engine input and comparators), §8b (bench, reference complexity), §9 (hint rules); `docs/requirements.md` PCK-1 to PCK-3, MCP-1, MCP-3, VAL-3, VAL-5, GHO-1, GHO-6; `.kiro/steering/grimoire-authoring.md`; `.kiro/specs/ghost-engine/` (engine types and fixtures).

## Scope constraints

- The only crate changed is `crates/grimoire/` (package `counterspell-grimoire`). It reuses `counterspell-engine` types (`Schema`, `Comparator`, `KnownGhost`, `CandidateStream`, `shrink`). It does not add a second schema or generator.
- Static validation is the default. It does not spawn processes, run Python, open sockets, or touch the database.
- Code execution (reference checks, known-ghost reproduction, timing) happens only through a `PackRunner` trait. The trusted judge implements it through its sandbox. The validator never falls back to the host.
- Pack files are untrusted data. The validator treats them the way the judge treats submissions.
- Hidden test inputs and hidden expected outputs never appear in diagnostics, in logs, or in the repository. Expected outputs for hidden tests come only from the Python reference.
- This spec does not import packs into Postgres, does not touch `migrations/`, and does not change `packs/_test/` (ghost-engine fixtures).
- The launch stack is fixed: Python and JavaScript at launch, with one Python reference per pack.

## Glossary

- **Pack**: a directory `packs/<slug>/` holding one problem.
- **Slug**: the directory name and the problem id, for example `count-pairs`.
- **Reference**: `reference/solution.py`, the authoritative Python implementation.
- **Incorrect program**: a deliberately wrong implementation under `incorrect/`, used to prove a known ghost.
- **Known ghost**: an authored failing input with its kind, expected/got values and Bug Autopsy text.
- **Hidden tests**: inputs the generator produces from `[generator]` seed. Their expected outputs are computed by the reference.
- **Static check**: a check that reads files only.
- **Sandbox check**: a check that runs code through `PackRunner`.
- **Starter pack**: one of the first original catalogue packs.

## Requirements

### A. Pack layout and file safety

1. **GRM-1** THE SYSTEM SHALL require these files in every pack: `problem.md`, `meta.toml`, `hints.toml`, `reference/solution.py`. It SHALL allow `known_ghosts.toml` and `incorrect/*.py` only when known ghosts exist.
2. **GRM-2** THE SYSTEM SHALL reject any other file in a pack, including dot-prefixed files, `.pyc` files, notebooks, archives and data files that are not in the allowed set.
3. **GRM-3** THE SYSTEM SHALL reject symbolic links, paths that resolve outside the pack root, and non-UTF-8 file names.
4. **GRM-4** THE SYSTEM SHALL enforce size caps: `problem.md` 32 KB, `meta.toml` 32 KB, `hints.toml` 8 KB, `known_ghosts.toml` 32 KB, each Python file 64 KB, and 512 KB for the whole pack.
5. **GRM-5** THE SYSTEM SHALL require every text file to be valid UTF-8 with no NUL byte and no control character other than tab, line feed and carriage return (the VAL-3 rule).
6. **GRM-6** THE SYSTEM SHALL require the directory name to match `^[a-z][a-z0-9-]{2,39}$` and to equal `slug` in `meta.toml`. Directories starting with `_` are validated the same way, but the importer skips them (GRM-45).
7. **GRM-7** THE SYSTEM SHALL never modify a pack while validating it.

### B. meta.toml

8. **GRM-8** THE SYSTEM SHALL parse `meta.toml` with unknown keys rejected at every level. Each parse error SHALL name the file and the line and column.
9. **GRM-9** THE SYSTEM SHALL require these fields: `slug`, `title` (1–80 characters), `status` (`draft` or `published`), `difficulty` (`easy`, `medium` or `hard`), `track` (kebab-case), `concepts` (1–5 kebab-case tags, unique), `origin` (must equal `"original"`), `author` (1–60 characters), `licence` (must equal `"MIT"`), `pack_version` (integer ≥ 1), `function`, `params`, `returns`, `comparator`, `limits`, `input_schema`, `generator`, `samples`.
10. **GRM-10** THE SYSTEM SHALL reject any metadata field that names a source URL or a third-party problem. Only original problems are accepted (PCK-3).
11. **GRM-11** THE SYSTEM SHALL accept `function` as a Python identifier (`^[a-z_][a-z0-9_]{0,63}$`). Each parameter has a unique `name` and a `type` from `int`, `str`, `list[int]`, `list[list[int]]`. `returns` is one of the same four types.
12. **GRM-12** THE SYSTEM SHALL require `input_schema` to be a `tuple` whose items match `params` one for one, in order, using this mapping: `int`→`int`, `str`→`str`, `list[int]`→`int_array`, `list[list[int]]`→`matrix`. The argument list is always the root tuple (system design §6 driver convention).
13. **GRM-13** THE SYSTEM SHALL run the engine's `Schema::check_definition` on `input_schema` and report its violations.
14. **GRM-14** THE SYSTEM SHALL accept `comparator` values `exact` and `unordered` only. `unordered` is allowed only when `returns` is `list[int]` or `list[list[int]]`.
15. **GRM-15** THE SYSTEM SHALL limit `limits.time_ms` to 100–2000 and `limits.mem_mb` to 64–256, the launch Python and JavaScript caps.
16. **GRM-16** WHERE `reference_complexity` is declared, THE SYSTEM SHALL require a `[bench]` table, and SHALL accept `reference_complexity` only from `1`, `log n`, `n`, `n log n`, `n^2`, `n^3`, `2^n` (the §8b fit model set).
17. **GRM-17** WHERE `[bench]` is declared, THE SYSTEM SHALL require `size_param` to name a parameter of type `int_array`, `str` or `matrix`, and `sizes` to be 3–8 strictly increasing integers with a maximum of 65536.
18. **GRM-18** THE SYSTEM SHALL cap `complexity_note` at 240 characters.

### C. Statement (problem.md)

19. **GRM-19** THE SYSTEM SHALL require the first line to be `# <title>` with the title equal to `meta.toml` `title`, followed by the level-2 headings `## Task`, `## Input`, `## Output`, `## Examples` and `## Constraints`, in that order.
20. **GRM-20** THE SYSTEM SHALL cap `problem.md` at 8000 characters and reject raw HTML tags (a `<` followed by a letter or `/`). The UI renders the statement as markdown with HTML disabled.

### D. Samples and hidden tests

21. **GRM-21** THE SYSTEM SHALL require 2–5 `[[samples]]`, each with an `input` that the input schema accepts and an `expected` value that matches the `returns` shape.
22. **GRM-22** THE SYSTEM SHALL reject duplicate sample inputs.
23. **GRM-23** THE SYSTEM SHALL require `[generator]` with `seed` (integer 0 to 2^63−1, because TOML integers are signed 64-bit) and `hidden_count` (5–50).
24. **GRM-24** THE SYSTEM SHALL check statically that the first 200 candidates from the engine's seeded stream satisfy the schema and are unique (PCK-1.2).
25. **GRM-25** THE SYSTEM SHALL never print hidden inputs or hidden expected outputs in diagnostics, JSON output or logs. Hidden data is generated in memory and shown, at most, as a count or a pointer.

### E. Hints

26. **GRM-26** THE SYSTEM SHALL require exactly four `[[hint]]` entries in `hints.toml`, numbered 1 to 4 in order (R1 nudge, R2 approach, R3 pseudo-code, R4 targeted fix).
27. **GRM-27** THE SYSTEM SHALL require each hint text to be 1–600 characters, with no fenced code block longer than 3 lines (system design §9).

### F. Known ghosts and Bug Autopsy text

28. **GRM-28** WHERE `known_ghosts.toml` exists, THE SYSTEM SHALL allow at most 10 `[[ghost]]` entries with unique kebab-case `id`s.
29. **GRM-29** THE SYSTEM SHALL require each ghost `input` to satisfy the schema, and its `program` to name an existing `incorrect/*.py` file that defines `function`.
30. **GRM-30** THE SYSTEM SHALL apply these rules to `kind`: `wrong` requires `got` (valid for `returns`) and forbids `exception_class`; `crash` requires `exception_class` matching `^[A-Za-z_][A-Za-z0-9_.]{0,63}$` and forbids `got`; `timeout` forbids both.
31. **GRM-31** THE SYSTEM SHALL require each authored `explanation` to be 1–400 bytes of plain text, with no phrase from the engine's forbidden list (`smallest`, `minimum`, `optimal`, and the others in the engine's R9.6 list). The explanation describes the mistake, not the optimality of the ghost.
32. **GRM-32** THE SYSTEM SHALL require each ghost `concepts` to be a subset of the pack's `concepts`.
33. **GRM-33** THE SYSTEM SHALL allow `dogfood = true` only in packs whose directory starts with `_` (MCP-3: the demo pack is not in the catalogue).

### G. Reference and programs

34. **GRM-34** THE SYSTEM SHALL require exactly one reference, `reference/solution.py`, and SHALL require it to contain a top-level `def <function>(` line.
35. **GRM-35** THE SYSTEM SHALL NOT interpret or execute Python in static mode. The "defines the function" rule is a text check, and the sandbox check is the real test.

### H. Diagnostics and CLI

36. **GRM-36** EVERY diagnostic SHALL carry a stable code (table in `design.md`), a severity (`error` or `warning`), the file, an optional JSON pointer or line, and a plain message.
37. **GRM-37** THE SYSTEM SHALL exit with 0 when there are no errors (and no warnings under `--deny-warnings`), 1 when there are validation errors, and 2 for usage or I/O failures.
38. **GRM-38** WHERE `--json` is given, THE SYSTEM SHALL print one JSON report on stdout with the same codes and fields. The MCP tool `validate_grimoire_pack` reuses this report.
39. **GRM-39** WHERE `--all <root>` is given, THE SYSTEM SHALL validate every pack directory under the root except `_`-prefixed ones, unless `--include-underscore` is also given.
40. **GRM-40** WHERE `--sandbox` is given, THE SYSTEM SHALL run the sandbox checks through the configured `PackRunner`. With no runner configured it SHALL exit 2 with the message "sandbox runner not configured". It SHALL NOT fall back to host execution.

### I. Sandbox checks (need the judge's PackRunner)

41. **GRM-41** THE SYSTEM SHALL check that the reference reproduces every `[[samples]]` `expected` value, using the engine comparator. A mismatch is an error. The validator SHALL NOT replace the authored value with the reference value.
42. **GRM-42** THE SYSTEM SHALL run the reference on every hidden input within `limits.time_ms`, and SHALL report an error if any run crashes, times out or returns the wrong shape. Hidden expected values stay in memory and are not written to disk.
43. **GRM-43** THE SYSTEM SHALL check that each known ghost reproduces its `kind`, `exception_class` or `got` on its `program` through the runner.
44. **GRM-44** WHERE a known ghost input is not minimal, THE SYSTEM SHALL warn and report the pointer and the size of the smaller failing input the engine found (GHO-6: ghost text must not claim minimality without evidence).
45. **GRM-45** WHERE `[bench]` is declared, THE SYSTEM SHALL time the reference through the runner and warn when the measured class differs from `reference_complexity`. Until the efficiency-familiar fit exists, THE SYSTEM SHALL emit warning `SBX-08` ("complexity not checked") instead of silently passing (PCK-1.3).
46. **GRM-46** IF the runner fails for infrastructure reasons, THEN THE SYSTEM SHALL exit 2 and report an infrastructure error. It SHALL NOT report a pack failure, mirroring "infrastructure failure ≠ wrong answer".

### J. Starter packs

47. **GRM-47** THE SYSTEM SHALL ship three starter packs, `count-pairs`, `longest-run-length` and `diagonal-total`. Each has `status = "draft"` until Brighton reviews its statement, then `published`. Each passes static validation, and each passes sandbox validation before it is marked published.
48. **GRM-48** Each starter pack SHALL include a full four-rung hint ladder, a Python reference, at least two known ghosts with authored Bug Autopsy explanations, and the incorrect programs they cite.
49. **GRM-49** Starter statements SHALL be written from scratch as original text. They SHALL NOT reproduce HackerRank, LeetCode or other third-party statements (PCK-3).
50. **GRM-50** THE SYSTEM SHALL keep the three ghost-engine fixtures under `packs/_test/` unchanged. Their conversion to this format is a separate task after ghost-engine ships.

### K. Tests, CI and boundary

51. **GRM-51** THE SYSTEM SHALL include a negative fixture for every diagnostic code, under `crates/grimoire/tests/fixtures/`, and each fixture SHALL fail with exactly its intended code.
52. **GRM-52** `cargo test -p counterspell-grimoire` SHALL statically validate every starter pack under `packs/`.
53. **GRM-53** THE CRATE SHALL contain no `unwrap()`, `expect()` or `unsafe` in production code, and SHALL pass `cargo clippy -D warnings`.
54. **GRM-54** THE CRATE SHALL expose a typed `ProblemRecord` (system design §4, `problems` columns: slug, title, difficulty, statement_md, signature, input_schema, comparator, limits, reference_complexity, bench, complexity_note, version, concepts) for the later import step. The crate SHALL NOT depend on sqlx or write to a database.
55. **GRM-55** THE SYSTEM SHALL NOT ship a keyword or API blocklist for pack code as a security control (security.md). Pack code is only ever run inside the sandbox.

## Out of scope

- Importing packs into Postgres, the catalogue API, and admin publish/unpublish (ADM-1, server spec).
- Judge `PackRunner` implementation (judge-sandbox spec).
- The bench fit model and the efficiency-familiar mood (efficiency-familiar spec).
- Languages other than Python and JavaScript, and Java or C++ drivers (language-drivers, stretch).
- The remaining 7 of the 10 launch problems (later specs, after the starter packs pass).
- The Kiro Power and the MCP tool wiring (counterspell-mcp specs).
- Float comparators, tree and graph schemas, and Unicode alphabets.
