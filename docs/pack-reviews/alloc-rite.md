# Pack review summary: alloc-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `alloc-rite` |
| Title | The Alloc Rite |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `alloc_rite` |
| Content hash | `390b891dea86dcf194818501d28807d74ffa772dca12b80fe3321ef9be5de5ec` |

## Mechanic

First-fit allocation — the first big-enough slot shrinks by the amount taken.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
