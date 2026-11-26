# Pack review summary: header-check

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `header-check` |
| Title | The Header Check |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `header_check` |
| Content hash | `b7f9ee2b82c8b3b993406f14b4cc5c89026be0a0a5bce3c0dbf5d80f5eabf6f5` |

## Mechanic

Allowlist header validation — required names present, no forbidden names.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
