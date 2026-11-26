# Pack review summary: prime-ward

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `prime-ward` |
| Title | The Prime Ward |
| Difficulty | medium |
| Track | `math` |
| Entry | `prime_ward` |
| Content hash | `272a3ec59d760353dc3049b0e335972e55be7df1be3989da6dc38b2fc6bac428` |

## Mechanic

Prime counting up to a cap — a sieve with small-bound edges.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
