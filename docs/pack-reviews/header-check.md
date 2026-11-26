# Pack review summary: header-check

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `header-check` |
| Title | The Header Check |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `header_check` |
| Content hash | `048cf083cfe50e1883de953b41bcc577c570ccb2d104396125a65c7ad1e374b2` |

## Mechanic

Protocol header validation — fields must sit inside their ranges.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
