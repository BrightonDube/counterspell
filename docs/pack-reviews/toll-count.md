# Pack review summary: toll-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `toll-count` |
| Title | The Toll Count |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `toll_count` |
| Content hash | `5194dafcfad1ecbc87fe0ffb8b252720feb39561f975888490a08ea651a35bc7` |

## Mechanic

Counts arcs priced below the toll cap.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
