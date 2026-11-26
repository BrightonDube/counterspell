# Pack review summary: null-join

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `null-join` |
| Title | The Null Join |
| Difficulty | medium |
| Track | `databases` |
| Entry | `null_join` |
| Content hash | `833fd88ce1cd1d058d2178d19eae4f4a467feacaa553996ab205e07dfe6aa95b` |

## Mechanic

Join where -1 keys never match -1 keys (NULL semantics) but still match real keys on the other side's rule.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
