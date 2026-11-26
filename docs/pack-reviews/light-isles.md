# Pack review summary: light-isles

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `light-isles` |
| Title | The Light Isles |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `light_isles` |
| Content hash | `ef9dd2b35e6f50f15fc904296b2987df8f4f197297cc53add6791a636e2ada3c` |

## Mechanic

Union-find isles over arcs whose price stays under the toll cap.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
