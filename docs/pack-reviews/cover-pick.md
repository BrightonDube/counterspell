# Pack review summary: cover-pick

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `cover-pick` |
| Title | The Cover Pick |
| Difficulty | hard |
| Track | `arrays` |
| Entry | `cover_pick` |
| Content hash | `240f03c025f8aef0e3676e5b01096ddedc998dca67ac9b09354b4a79b2bed7b6` |

## Mechanic

Greedy bitmask set cover — bits meet greedy picks.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
