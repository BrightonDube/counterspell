# Pack review summary: key-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `key-count` |
| Title | The Key Count |
| Difficulty | easy |
| Track | `databases` |
| Entry | `key_count` |
| Content hash | `b140a664e2016440a233e54caa067044f7bb2cb0f73c66526bcab95a910c45e8` |

## Mechanic

Distinct-key tally over a key column that may repeat or carry a -1 sentinel.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
