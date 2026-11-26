# Pack review summary: route-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `route-count` |
| Title | The Route Count |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `route_count` |
| Content hash | `4ad04bcb02882c50ae9bd715682a1a38049eb7d3484de3926018fa6304966e81` |

## Mechanic

Reachable-node count on a bounded directed graph — BFS from the origin ward.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
