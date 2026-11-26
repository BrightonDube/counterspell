# Pack review summary: null-marks

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `null-marks` |
| Title | The Null Marks |
| Difficulty | easy |
| Track | `databases` |
| Entry | `null_marks` |
| Content hash | `6cddbf0782d82f0c61eb843d88e48b5fc9deec57681eaf68b852c3a3cc93836d` |

## Mechanic

Counting non-null marks in a column where -1 stands in for NULL.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
