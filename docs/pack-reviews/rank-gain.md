# Pack review summary: rank-gain

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rank-gain` |
| Title | The Rank Gain |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `rank_gain` |
| Content hash | `c19eefc3f81524be0157b35e2db3160df488d87c4518b6c4cc2410168c33399a` |

## Mechanic

Linearly decayed rank gain — position-weighted prefix.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
