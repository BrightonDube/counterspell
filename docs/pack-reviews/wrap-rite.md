# Pack review summary: wrap-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `wrap-rite` |
| Title | The Wrap Rite |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `wrap_rite` |
| Content hash | `0ef477e3a4ea332ee900c7346091c560368cec48d9d2ca2f3cf3617e0fc7ba00` |

## Mechanic

Ring-buffer overwrites — a stranger on a full ring evicts the oldest; repeats cost nothing.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
