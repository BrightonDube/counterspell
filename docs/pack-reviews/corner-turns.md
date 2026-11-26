# Pack review summary: corner-turns

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `corner-turns` |
| Title | The Corner Turns |
| Difficulty | hard |
| Track | `math` |
| Entry | `corner_turns` |
| Content hash | `4618dacea51ecbc31f57be05f102d5451bfa007725c77d48ea286a63af6cdd44` |

## Mechanic

Left-turn counting along a polyline via signed cross products.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
