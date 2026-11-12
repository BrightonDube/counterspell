# Problem packs (Grimoire format)

Status: **provisional until grimoire-packs is reviewed.** This file describes the format the validator (`counterspell-grimoire validate`) checks. The rules here are the same as the codes in `crates/grimoire/src/codes.rs`.

Sources: `docs/system-design.md` §4 (problem columns), §6 (languages), §8 (engine input and comparators), §8b (bench and reference complexity), §9 (hint rules). The requirements are in `docs/requirements.md` (PCK-1 to PCK-3).

`packs/` holds reviewed files only. There is no code in this folder except the Python reference and incorrect programs inside each pack. Nothing here is run by the validator in its default mode.

## Layout

```text
packs/
├── README.md                  this file
├── count-pairs/               one pack = one problem; directory name is the slug
│   ├── problem.md             statement (required)
│   ├── meta.toml              metadata, signature, schema, generator, samples (required)
│   ├── hints.toml             four-rung hint ladder (required)
│   ├── known_ghosts.toml      authored failing inputs (optional)
│   ├── reference/solution.py  the Python reference (required, exactly one)
│   └── incorrect/*.py         deliberately wrong programs (only with known_ghosts.toml)
└── _test/                     ghost-engine fixtures; underscore packs are skipped by the importer
```

## Required and optional files

- **Required:** `problem.md`, `meta.toml`, `hints.toml`, `reference/solution.py`.
- **Optional:** `known_ghosts.toml` and `incorrect/*.py`. Both are allowed only together.
- **Nothing else.** Any other file is rejected, including dot-prefixed files, `.pyc` files, notebooks, archives and data files.
- **Directory names** start with a letter, use lower-case letters, digits and hyphens, and are 3 to 40 characters long. A name that starts with `_` is for tests only.

## Hidden data

The authoring process never writes generated hidden inputs or hidden expected outputs to any file. The pack commits the seed and the reference. The judge creates the hidden tests in memory. Diagnostics show counts and pointers, never values. Open question 1 in `.kiro/specs/grimoire-packs/design.md` asks Brighton to confirm that the reference and seed stay in git.

## Size caps

| File | Cap |
|---|---|
| `problem.md` | 32 KB, and 8000 characters |
| `meta.toml` | 32 KB |
| `hints.toml` | 8 KB |
| `known_ghosts.toml` | 32 KB |
| each Python file | 64 KB |
| whole pack | 512 KB |

## Text rules

Every text file is valid UTF-8. It has no NUL byte and no control character except tab, line feed and carriage return. The `problem.md` statement may not contain raw HTML tags.

## `meta.toml` fields

| Field | Rule |
|---|---|
| `slug` | equals the directory name |
| `title` | 1 to 80 characters |
| `status` | `draft` or `published` |
| `difficulty` | `easy`, `medium` or `hard` |
| `track` | kebab-case |
| `concepts` | 1 to 5 unique kebab-case tags |
| `origin` | must be `"original"` |
| `author` | 1 to 60 characters |
| `licence` | must be `"MIT"` |
| `pack_version` | integer, 1 or more |
| `function` | Python identifier, up to 64 characters |
| `params` | unique names; types `int`, `str`, `list[int]`, `list[list[int]]` |
| `returns` | one of the same four types |
| `comparator` | `exact`, or `unordered` for list returns only |
| `limits` | `time_ms` 100 to 2000, `mem_mb` 64 to 256 |
| `input_schema` | a `tuple` with one item per parameter, in order |
| `generator` | `seed` and `hidden_count` |
| `samples` | 2 to 5 `[[samples]]` |

Unknown keys are rejected at every level. A source URL or a third-party problem field is rejected. Only original problems are accepted.

Parameter type to schema kind: `int` to `int`, `str` to `string`, `list[int]` to `int_array`, `list[list[int]]` to `matrix`. The root of `input_schema` is always the argument tuple.

## Optional `[bench]` and `reference_complexity`

`reference_complexity` is one of `1`, `log n`, `n`, `n log n`, `n^2`, `n^3`, `2^n`. It needs a `[bench]` table. `[bench]` needs `size_param` naming an `int_array`, `str` or `matrix` parameter, and `sizes` with 3 to 8 strictly increasing integers up to 65536. `complexity_note` is at most 240 characters. Starter packs do not declare these yet.

## `problem.md`

- The first line is `# <title>`, and the title equals `meta.toml` `title`.
- Five level-2 headings follow in order: `## Task`, `## Input`, `## Output`, `## Examples`, `## Constraints`.
- At most 8000 characters. Markdown is rendered with HTML off.

## `hints.toml`

Exactly four `[[hint]]` entries, with `rung` 1, 2, 3, 4 in order:

1. nudge
2. approach
3. pseudo-code
4. targeted fix

Each text is 1 to 600 characters. A fenced code block inside a hint is at most 3 lines (system design §9).

## Samples and hidden tests

- Each `[[samples]]` entry has an `input` (the argument list in parameter order) and an `expected` value. The input must fit the schema, and the expected value must match `returns`.
- Two samples cannot have the same input.
- `[generator]` has `seed` (0 to 2^63 - 1) and `hidden_count` (5 to 50). The engine draws the first 200 candidates from the seed. Each must fit the schema, and none may repeat.

## Known ghosts (optional)

`known_ghosts.toml` holds at most 10 `[[ghost]]` entries with unique kebab-case `id` values.

- `input` must fit the schema.
- `program` names a file under `incorrect/`, and that file defines `function`.
- `kind` `wrong` needs `got` and forbids `exception_class`. `kind` `crash` needs `exception_class` (a dotted Python name) and forbids `got`. `kind` `timeout` forbids both.
- `explanation` is 1 to 400 bytes of plain text. It describes the mistake. It may not claim the ghost is smallest, minimum or optimal.
- Each ghost's `concepts` must be a subset of the pack's `concepts`.
- `dogfood = true` is allowed only in `_` packs.

## Reference

Exactly one `reference/solution.py`. It must contain a line starting at column 0 with `def <function>(`. The static check reads the text and does not run Python. The sandbox check proves the behaviour.

## Diagnostic codes

Each code has a stable name and a severity. A code appears in one module only (`codes.rs`).

| Code | Severity | Plain-English rule |
|---|---|---|
| LAY-01 | error | A required file is missing. |
| LAY-02 | error | A file is outside the allow-list, or its name starts with a dot. |
| LAY-03 | error | A file is a symbolic link, resolves outside the pack, or has a non-UTF-8 name. |
| LAY-04 | error | A file or the whole pack is over its size cap. |
| LAY-05 | error | A text file is not UTF-8, or contains a NUL or a forbidden control character. |
| LAY-06 | error | The directory name is not a valid slug, or differs from `meta.toml` `slug`. |
| MET-01 | error | `meta.toml` has a TOML syntax error or an unknown key (file, line and column given). |
| MET-02 | error | A required field is missing, has the wrong type, or is out of range. |
| MET-03 | error | `origin` is not `"original"`, `licence` is not `"MIT"`, or a third-party field exists. |
| MET-04 | error | A parameter or return type is not in the launch set. |
| MET-05 | error | `input_schema` does not match `params` in count or kind. |
| MET-06 | error | The engine's `check_definition` rejects the schema. |
| MET-07 | error | The comparator is not allowed for this return type. |
| MET-08 | error | `limits` are outside the launch caps. |
| MET-09 | error | `reference_complexity` or `[bench]` is invalid, or only one of them is present. |
| STM-01 | error | The title line or the heading order in `problem.md` is wrong. |
| STM-02 | error | The statement is too long, or contains raw HTML. |
| SMP-01 | error | The sample count is not 2 to 5. |
| SMP-02 | error | A sample input fails the schema, or its expected value does not match `returns`. |
| SMP-03 | error | Two samples have the same input. |
| GEN-01 | error | The generator seed or hidden count is out of range. |
| GEN-02 | error | The engine's first 200 candidates include an invalid or repeated value. |
| HNT-01 | error | `hints.toml` does not have exactly four hints in rung order 1 to 4. |
| HNT-02 | error | A hint is too long, or has a code fence longer than 3 lines. |
| KNG-01 | error | There are more than 10 known ghosts, or an `id` is duplicated or not kebab-case. |
| KNG-02 | error | A ghost input fails the schema, or its `program` is missing or does not define the function. |
| KNG-03 | error | A ghost's `kind` does not match its `got` or `exception_class` fields. |
| KNG-04 | error | A ghost explanation is empty, too long, or uses a forbidden phrase. |
| KNG-05 | error | A ghost concept is not in the pack's `concepts`. |
| KNG-06 | error | `dogfood = true` appears in a pack whose name does not start with `_`. |
| REF-01 | error | The reference is missing, or does not contain a column-0 `def <function>(` line. |
| SBX-01 | error | The reference disagrees with an authored sample. |
| SBX-02 | error | The reference crashes, times out, or returns the wrong shape on a hidden input. |
| SBX-03 | error | A known ghost does not reproduce its failure on its program. |
| SBX-04 | warning | A known ghost input is not minimal; a smaller failing input was found. |
| SBX-05 | warning | The measured complexity differs from `reference_complexity`. |
| SBX-06 | error | A runner infrastructure failure occurred (exit 2, not a pack failure). |
| SBX-07 | error | No sandbox runner is configured (exit 2). |
| SBX-08 | warning | Complexity is not checked yet, because the bench fit is not available. |

Sandbox checks need a configured runner. Without one, `--sandbox` exits with code 2 (SBX-07). Infrastructure failures are never reported as pack failures (SBX-06).

## Validation

```text
cargo run -q -p counterspell-grimoire -- validate packs/count-pairs
cargo run -q -p counterspell-grimoire -- validate --all packs
```

Exit codes: 0 means no errors. 1 means validation errors. 2 means a usage, I/O or infrastructure failure. `--deny-warnings` also makes warnings fail with code 1. `--json` prints one report with the same codes.

## Status and publication

Every starter pack starts as `status = "draft"`. Brighton reviews the statement, hints, ghost explanations and reference, then changes the status to `published`. Agents do not publish packs.
