# Pack review summary: leak-check

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `leak-check` |
| Title | The Leak Check |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `leak_check` |
| Content hash | `4516b75b2948c682eaea9db815c01e9293838591da98754b7f7fa19cde342455` |

## Mechanic

Train/test contamination — the distinct-id intersection.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
