# Pack review summary: frame-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `frame-rite` |
| Title | The Frame Rite |
| Difficulty | hard |
| Track | `arrays` |
| Entry | `frame_rite` |
| Content hash | `8d51ab723740b65f3d2b7cb075ac2008420a09f7798816db8cfb09dd63c9c7c8` |

## Mechanic

Optimal page replacement — evict the resident whose next use is farthest, leftmost on ties.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
