# Pack review summary: toll-hop

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `toll-hop` |
| Title | The Toll Hop |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `hop_toll` |
| Content hash | `8159f7719d4364f185428c769b2d693293311664c2f9c545a971384b621cae8a` |

## Mechanic

State-expanded Dijkstra over (node, hops used) under a hop cap.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
