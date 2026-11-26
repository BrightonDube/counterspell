# Pack review summary: vector-clock

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `vector-clock` |
| Title | The Vector Clock |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `clock_cmp` |
| Content hash | `bc03dc530808ba147abc07ee296f1eecb0b78e2c559d2dc608ecdeb2eef7dca5` |

## Mechanic

Component-wise vector-clock merge for causality tracking.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
