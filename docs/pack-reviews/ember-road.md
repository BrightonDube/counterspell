# Pack review summary: ember-road

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `ember-road` |
| Title | The Ember Road |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `road_toll` |
| Content hash | `2af9189a7dd53f28bbaa1f114316564adc044e95c81028ae7ee01ce0bccf34b3` |

## Mechanic

Lazy Dijkstra over a priced web; -1 when the target is unreachable.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
