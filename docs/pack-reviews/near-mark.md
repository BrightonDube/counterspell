# Pack review summary: near-mark

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `near-mark` |
| Title | The Near Mark |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `near_mark` |
| Content hash | `a88f25c7edc5f1bfaaffb907790e7848739a47d2ebe2d369e89a5f34bea3a778` |

## Mechanic

Cheapest arc leaving node 0 over a priced web; -1 when none leaves.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
