# Pack review summary: ebb-road

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `ebb-road` |
| Title | The Ebb Road |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `ebb_toll` |
| Content hash | `ec41cb93e99dfc2a97ec38fb5ee30748a9f4bc8046573aa49d521a148fb28714` |

## Mechanic

Bellman-Ford over one-way arcs with negative prices; ring sentinel via marked reachability.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
