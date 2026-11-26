# Pack review summary: rate-gate

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rate-gate` |
| Title | The Rate Gate |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `rate_gate` |
| Content hash | `9170c180b014bf609a0d88284ab59b30521652e1374b30384d417c8ccdad3024` |

## Mechanic

Token-bucket gate — accepted versus throttled request counts.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
