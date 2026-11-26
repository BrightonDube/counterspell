# Pack review summary: chunk-toll

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `chunk-toll` |
| Title | The Chunk Toll |
| Difficulty | hard |
| Track | `arrays` |
| Entry | `chunk_toll` |
| Content hash | `6c3667ec6e153c51403afe7ac584b2e2846c95c4d457b39ea4b1d1ea7efe6778` |

## Mechanic

Windowed partition DP — base fee plus chunk spread, minimised.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
