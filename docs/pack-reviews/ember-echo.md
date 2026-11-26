# Pack review summary: ember-echo

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `ember-echo` |
| Title | The Ember Echo |
| Difficulty | medium |
| Track | `strings` |
| Entry | `echo_len` |
| Content hash | `20ae0c78319ec88733445908a2eb364f1116392f2feec6912e5cf30fec3f5917` |

## Mechanic

Longest shared echo between two scrawls — the paired-prefix table.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
