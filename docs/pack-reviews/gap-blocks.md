# Pack review summary: gap-blocks

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `gap-blocks` |
| Title | The Gap Blocks |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `block_count` |
| Content hash | `e1b8a1c7b379127e0d0417a1027f48adb8b1d0a9829cdf27cabc08258b0c99ea` |

## Mechanic

Count of merged runs after sorting — the interval-merge block count.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
