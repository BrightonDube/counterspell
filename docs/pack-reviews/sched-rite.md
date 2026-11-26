# Pack review summary: sched-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `sched-rite` |
| Title | The Sched Rite |
| Difficulty | hard |
| Track | `databases` |
| Entry | `sched_rite` |
| Content hash | `5043c9b2e48957c402e61211da334b0bd4d25ea0e91bcb8f52ed8f1d04370bee` |

## Mechanic

Precedence-graph acyclicity over read/write edges between transactions in a schedule.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
