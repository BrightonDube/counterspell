# Pack review summary: rate-gate

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rate-gate` |
| Title | The Rate Gate |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `rate_gate` |
| Content hash | `b8ff55ba592b2b5d9eecd663b15da39081e28edf56512ffb7ff95f9643fce9d8` |

## Mechanic

Token-bucket admission — accept while tokens remain, refill by rate.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
