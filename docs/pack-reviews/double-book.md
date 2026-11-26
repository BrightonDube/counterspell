# Pack review summary: double-book

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `double-book` |
| Title | The Double Book |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `room_count` |
| Content hash | `e6cd22cd5959b1aad767f2977251b4aae8b79017247aff96b1ee17938fccbb44` |

## Mechanic

Interval partitioning — deepest overlap via an event sweep.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
