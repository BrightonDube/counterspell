# Pack review summary: mask-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `mask-rite` |
| Title | The Mask Rite |
| Difficulty | hard |
| Track | `math` |
| Entry | `mask_rite` |
| Content hash | `9b9e7c2291a2886d052343dc79f702d52780b4afdad15f768314cf4f374bad6e` |

## Mechanic

Subset counting by bitmask enumeration under a cap.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
