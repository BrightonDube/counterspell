# Pack review summary: coin-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `coin-rite` |
| Title | The Coin Rite |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `coin_rite` |
| Content hash | `fc98340270fbafeb3da6f2d52a1d1dd9ca3ce6e53cdf0e4ba1bb5268a351efc9` |

## Mechanic

Unbounded fewest coins — bottom-up min over last-kind choices.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
