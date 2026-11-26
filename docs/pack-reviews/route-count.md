# Pack review summary: route-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `route-count` |
| Title | The Route Count |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `route_count` |
| Content hash | `9323bec8ea43eb28da47999f608dec6b22b07d70031a72e1db184db19358cf17` |

## Mechanic

BFS reachability over a small grid of towers — count reachable marks from the origin.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
