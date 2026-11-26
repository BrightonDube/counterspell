# Pack review summary: relay-gap

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `relay-gap` |
| Title | The Relay Gap |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `relay_count` |
| Content hash | `4c9be1f90dd0298b29a37942b1f095e1dfa4063643cba34c8f70c92e7a565154` |

## Mechanic

Activity selection with a mandatory gap between picks.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
