# Pack review summary: serial-check

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `serial-check` |
| Title | The Serial Check |
| Difficulty | hard |
| Track | `databases` |
| Entry | `serial_check` |
| Content hash | `84353bd478c008eb079437efefbe803fb39d60f9a26a9245d46939de27034b70` |

## Mechanic

Serialisability of contiguous transactions: each id's stretch must not interleave with a repeated id.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
