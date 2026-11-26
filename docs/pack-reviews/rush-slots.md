# Pack review summary: rush-slots

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `rush-slots` |
| Title | The Rush Slots |
| Difficulty | medium |
| Track | `matrices` |
| Entry | `pick_count` |
| Content hash | `4b0b60de0abb27889b24da6a6c6ebac14c7332a1e009a7a5e326fafe050d58a9` |

## Mechanic

Activity selection — earliest end first, strict-after picks.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
