# Pack review summary: nest-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `nest-count` |
| Title | The Nest Count |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `nest_pairs` |
| Content hash | `ce1f3c2bd581a5d65b9accf47b4ea1158885a87711fd2ba660b1660092cacf84` |

## Mechanic

Strict containment pair counting over seals.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
