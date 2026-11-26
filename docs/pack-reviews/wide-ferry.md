# Pack review summary: wide-ferry

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `wide-ferry` |
| Title | The Wide Ferry |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `ferry_load` |
| Content hash | `b389c509d143f209744942fb11ebf93ab8ad7552d0abe50ca4baaca35cf892ba` |

## Mechanic

Max-heap maximin walk — largest weakest-arc capacity to the target.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
