# Pack review summary: group-tally

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `group-tally` |
| Title | The Group Tally |
| Difficulty | medium |
| Track | `databases` |
| Entry | `group_tally` |
| Content hash | `57005f4daecad125ba6dac6e73fdad8b8de2bcd13a193e76772beb0ec5e71e06` |

## Mechanic

Group-by sum with a threshold filter: totals per key, only groups meeting the floor counted.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
