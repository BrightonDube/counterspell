# Pack review summary: token-window

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `token-window` |
| Title | The Token Window |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `token_window` |
| Content hash | `dc9f8fdc8d9e6d9894a3603ce80f522a4d47bf02b5b097feff790c48eec845d8` |

## Mechanic

Sliding-window mark density over a small stream.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
