# Pack review summary: chain-toll

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `chain-toll` |
| Title | The Chain Tolls |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `toll_ways` |
| Content hash | `591ed14bad2fe508fcf02bf03a1b3c5949bd50447ddad943cb31ca40bda73250` |

## Mechanic

Dijkstra with a ways ledger counting distinct cheapest routes.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
