# Pack review summary: vault-split

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `vault-split` |
| Title | The Vault Split |
| Difficulty | hard |
| Track | `arrays` |
| Entry | `split_diff` |
| Content hash | `84e4e6adecf0dbdfb749468d0e740e4d22ac44923a9b812063bc29680c498579` |

## Mechanic

Closest two-way split — a reachable set under the half-sum.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
