# Pack review summary: rank-gain

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rank-gain` |
| Title | The Rank Gain |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `rank_gain` |
| Content hash | `d6e2678ad7ae8c15ba366513477ef7a293aa7fe61153c3ac1aacc04003cc598b` |

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
