# Pack review summary: split-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `split-rite` |
| Title | The Split Rite |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `split_rite` |
| Content hash | `9b961177563f86f9c26885b38bad39168a71d6bbf5f3a04a2c9161bedfe1659f` |

## Mechanic

Partition-aware quorum — which isle still holds a strict majority after splits.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
