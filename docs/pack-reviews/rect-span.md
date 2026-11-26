# Pack review summary: rect-span

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rect-span` |
| Title | The Rect Span |
| Difficulty | medium |
| Track | `math` |
| Entry | `rect_span` |
| Content hash | `e81651e98cfaa31af2cfcb402736304086e8eaa392e4ac94a6983d98413f0ef4` |

## Mechanic

Axis-aligned overlap area with unordered corners.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
