# Pack review summary: cover-pick

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `cover-pick` |
| Title | The Cover Pick |
| Difficulty | hard |
| Track | `arrays` |
| Entry | `cover_pick` |
| Content hash | `4179f6d6b7e136495cc4135a142f31437a571c130243f287a9ad8d6239e8d50c` |

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
