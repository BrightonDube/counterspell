# Pack review summary: ember-words

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `ember-words` |
| Title | The Ember Words |
| Difficulty | hard |
| Track | `strings` |
| Entry | `word_break` |
| Content hash | `d9b1f5ea07e522a1f4911146a5514dc2f47d8fe03fae73ebefe835a793a76b01` |

## Mechanic

Dictionary word break — prefix reachability over ember ends.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
