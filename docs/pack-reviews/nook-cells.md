# Pack review summary: nook-cells

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `nook-cells` |
| Title | The Nook Cells |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `nook_ways` |
| Content hash | `72dc1168e5ed09eefc165133ea4ece5bb54df7b44bd84898db3f20eb23c18892` |

## Mechanic

Path counting with walled cells — reachable sums over open lanes.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
