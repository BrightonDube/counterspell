# Pack review summary: scrawl-edit

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `scrawl-edit` |
| Title | The Scrawl Edit |
| Difficulty | medium |
| Track | `strings` |
| Entry | `edit_toll` |
| Content hash | `21ebb0636f8290232b306ea9e15c36bf45509fe3844654207610417cca0e3063` |

## Mechanic

Fewest letter mends between scrawls — three transitions per cell.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
