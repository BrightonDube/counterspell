# Pack review summary: lock-order

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `lock-order` |
| Title | The Lock Order |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `lock_order` |
| Content hash | `916a71d3a025c3424cfe383bbffa0db64f2d0942b2475324817bc2a33bc9238f` |

## Mechanic

Deadlock-order check over a flat ward tape plus per-rite lengths: shared pairs must keep one direction.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
