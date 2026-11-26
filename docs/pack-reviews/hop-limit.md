# Pack review summary: hop-limit

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `hop-limit` |
| Title | The Hop Limit |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `hop_limit` |
| Content hash | `bf95779dce663a9ef395b1ed92c2977b4d4c40528892db24e57b51bc1b8e42c3` |

## Mechanic

TTL-bounded BFS — reachable marks within the hop limit.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
