# Pack review summary: first-free

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `first-free` |
| Title | The First Free Mark |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `first_free` |
| Content hash | `b3b41925a5f3d72d8a2a8f0724b62d69ad0cdf774d5a2a3bd2057f9ee5d90928` |

## Mechanic

Smallest uncovered integer mark — a mex over merged stretches.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
