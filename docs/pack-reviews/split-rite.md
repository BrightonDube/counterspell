# Pack review summary: split-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `split-rite` |
| Title | The Split Rite |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `split_rite` |
| Content hash | `3d2e6abc9f4367ce23e984ca5a0fa0467801fd24104ac7ef2376d7eb3adbf98f` |

## Mechanic

Post-failure partition quorum — count components holding strict majority.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
