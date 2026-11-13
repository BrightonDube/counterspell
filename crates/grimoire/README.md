# counterspell-grimoire

The pack validator for `packs/<slug>/`. Format rules are in `packs/README.md`. This note is for the server and judge specs.

## Entry points

| Item | Use |
|---|---|
| `validate_static(root) -> io::Result<Report>` | Static checks only. Reads files and never runs them. Used by the CLI and by the MCP tool later. |
| `validate_sandbox(root, &mut dyn PackRunner) -> Result<Report, SandboxFault>` | Static checks, then SBX-01 to SBX-04 through the runner. The sandbox stage runs only when the static checks have no errors. |
| `record::build(root) -> Result<ProblemRecord, RecordError>` | The import handoff. Built only from a pack with zero errors. The crate has no database types. |
| `PackRunner` (trait), `Target`, `RunnerError` | Implemented by the judge. Runner faults are `RunnerError`, never a `ProgramResult`. |
| `SandboxFault::diagnostic()` | The SBX-06 diagnostic for a runner or I/O fault (exit 2). |
| `codes::ALL` | Every diagnostic code, with its severity. Modules must not invent codes. |

The CLI is `counterspell-grimoire validate <pack-dir> [--json] [--deny-warnings] [--sandbox]` and `validate --all <packs-root> [--include-underscore]`. Exit codes: 0 pass, 1 errors (or warnings under `--deny-warnings`), 2 usage, I/O or infrastructure.

## Current state

- Static checks (LAY, MET, STM, SMP, GEN, HNT, KNG, REF) are complete and tested.
- Sandbox checks (SBX-01 to SBX-04) are tested with a model runner in `tests/sandbox_fake.rs`. That proves the logic, not the sandbox.
- SBX-08 is emitted for any `[bench]` table. `PackRunner::bench` is not called.
- `--sandbox` on the CLI prints SBX-07 and exits 2, because no runner is configured.
- Starter packs `count-pairs`, `longest-run-length` and `diagonal-total` are `draft` and validate with zero errors.

## Blocked items

- **T20, judge implements `PackRunner` and wires `--sandbox`.** Owned by the `judge-sandbox` spec. The judge must re-run the vectors in `sandbox_fake.rs` through the real Python files.
- **T19, exit 2 with SBX-06 from the CLI.** Waits for T20, because the CLI has no runner yet.
- **T28, publish the starter packs.** Human review by Brighton. Agent work stops at `draft`.
- **T29, convert `packs/_test/`.** Waits for the ghost-engine decision (open question 11).
- **T30, CI step `validate --all packs --deny-warnings`.** Waits for the CI spec.

## Decisions needed before import

1. **Open question 1:** are the reference solution and hidden-test seed public in git? The current packs assume yes.
2. **Open question 7:** `PackRunner` lives in grimoire and the judge depends on grimoire. Confirm before the judge spec starts.

## Known engine issue

`counterspell-engine` panics in `tiny_ints` when a schema has min above max. The validator guards against this (`generator::check` stops on an invalid schema). The ghost-engine owner should make the engine return an error, not panic, before any MCP or judge path passes unchecked schemas to it.
