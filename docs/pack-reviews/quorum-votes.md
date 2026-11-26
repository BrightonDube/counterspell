# Pack review summary: quorum-votes

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `quorum-votes` |
| Title | The Quorum Votes |
| Difficulty | easy |
| Track | `math` |
| Entry | `quorum_reached` |
| Content hash | `4566144474083ff6ed154c72691b226299fd39b705e22c8f3f62a6c53798aa53` |

## Mechanic

Strict-majority quorum arithmetic over cast ballot weight.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
