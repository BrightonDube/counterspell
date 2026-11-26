# Pack review summary: nonce-ward

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `nonce-ward` |
| Title | The Nonce Ward |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `nonce_ward` |
| Content hash | `b204f224425ccf4be5fdeccc66b62147c1bca30450309ffa6930315264a17376` |

## Mechanic

Anti-replay nonce ledger — rejecting duplicate stamps within a window.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
