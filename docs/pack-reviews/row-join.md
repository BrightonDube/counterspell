# Pack review summary: row-join

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `row-join` |
| Title | The Row Join |
| Difficulty | easy |
| Track | `databases` |
| Entry | `row_join` |
| Content hash | `25116376dde2e4d5c86437dcb086b6cbe604f053abda17eca709c9545b15a71e` |

## Mechanic

Inner-join row count over two key-ordered tables: shared keys multiply paired rows.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
