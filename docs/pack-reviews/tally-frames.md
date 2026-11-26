# Pack review summary: tally-frames

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `tally-frames` |
| Title | The Tally Frames |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `tally_frames` |
| Content hash | `670fbe014363c22d139a665ff4ae916f97794809852fd6daabf0746219ef44f8` |

## Mechanic

Confusion-frame accuracy — the diagonal sum.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
