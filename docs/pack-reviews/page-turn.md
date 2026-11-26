# Pack review summary: page-turn

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `page-turn` |
| Title | The Page Turn |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `page_turn` |
| Content hash | `97e1677d90601b882a09762973b08d88b4b0288608cb114fb125ad3393a45917` |

## Mechanic

FIFO page faults — the folio forgets the oldest arrival.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
