# Pack review summary: wood-span

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `wood-span` |
| Title | The Wood Span |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `span_toll` |
| Content hash | `924aec6f5d2dcbf95fcf2a6e83f664becf6766fbae199f115bde9cc4c7b93464` |

## Mechanic

Kruskal-style sorted-arc union keeping the lowest joining toll; -1 when unjoinable.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
