# Pack review summary: idle-stretch

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `idle-stretch` |
| Title | The Idle Stretch |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `idle_stretch` |
| Content hash | `ca7ef986af232766f8f5337d3e26fbbae86c8147001263b6e7d92fc35011d26f` |

## Mechanic

Longest uncovered run inside a bounded watch.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
