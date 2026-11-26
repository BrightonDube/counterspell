# Pack review summary: retry-dedup

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `retry-dedup` |
| Title | The Retry Dedup |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `retry_dedup` |
| Content hash | `efec49759156019e4722adbea176de5224e4ff624eb1a2f73afc4eed64302818` |

## Mechanic

Idempotent retry dedup — counting distinct accepted keys across delivery attempts.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
