# Pack review summary: candle-decode

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `candle-decode` |
| Title | The Candle Decode |
| Difficulty | medium |
| Track | `strings` |
| Entry | `decode_ways` |
| Content hash | `0c017acea1c97ce0cbddaf7a1c9d9c6c8c713c3cff76e4f335b7e4a2bcd11a09` |

## Mechanic

Digit-line decoding with single- and pair-glyph steps; zero is the trap.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
