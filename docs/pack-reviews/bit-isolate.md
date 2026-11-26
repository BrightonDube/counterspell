# Pack review summary: bit-isolate

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `bit-isolate` |
| Title | The Bit Isolate |
| Difficulty | medium |
| Track | `math` |
| Entry | `bit_isolate` |
| Content hash | `3b02fa76957c19f5f40bd358879b40e1c671c4df5a8a1ea3e8b487e2d1841f36` |

## Mechanic

Lowest-set-bit extraction with a genuine zero contract.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
