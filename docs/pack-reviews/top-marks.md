# Pack review summary: top-marks

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `top-marks` |
| Title | The Top Marks |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `top_marks` |
| Content hash | `34e0dfabf6bd30a0d87c6886b11ad9ffa5f0ee6b196ca5d0f7b1d739a80a99b7` |

## Mechanic

Sum of the k largest marks — descending sort then prefix.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
