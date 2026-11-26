# Pack review summary: grille-toll

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `grille-toll` |
| Title | The Grille Toll |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `toll_grid` |
| Content hash | `94b6bcf87a01be12efe2a214030f52536f7d30aa8a08eb8e9724ff23b9df9b88` |

## Mechanic

Fewest cells across a walled grille — the min table with dead lanes.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
