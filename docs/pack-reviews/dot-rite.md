# Pack review summary: dot-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `dot-rite` |
| Title | The Dot Rite |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `dot_sparse` |
| Content hash | `e50e5e688dae7410d2035a61702238e66a501528294eeb246e62aec519757b48` |

## Mechanic

Sparse dot product — dict merge over shared position keys.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
