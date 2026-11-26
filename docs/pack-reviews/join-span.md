# Pack review summary: join-span

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `join-span` |
| Title | The Join Span |
| Difficulty | medium |
| Track | `databases` |
| Entry | `join_span` |
| Content hash | `6c9dfe9925c186f2cd0aedac0c00004e26e6e2e443bb453eb94d2abbbbb650b7` |

## Mechanic

Range join: pairing rows whose keys lie within a span and summing the carried values.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
