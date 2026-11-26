# Pack review summary: retry-dedup

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `retry-dedup` |
| Title | The Retry Dedup |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `retry_dedup` |
| Content hash | `e18c16c300d572a60994a0061379750faa726dc3c4d3f78284b84e0e4cde48f1` |

## Mechanic

Retry deduplication — count distinct rite marks in a retried log.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
