# Pack review summary: rune-toll

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rune-toll` |
| Title | The Rune Toll |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `rune_toll` |
| Content hash | `c4a602ed57d004040d3588334ca0fc220f1c23ad08c3ab66503cfb75c334999b` |

## Mechanic

Sums the priced arcs over a weighted web; out-of-range endpoints are skipped.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
