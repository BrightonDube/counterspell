# Pack review summary: miss-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `miss-count` |
| Title | The Miss Count |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `miss_count` |
| Content hash | `26f94961c93bcf3695516161faf76adec2837f560c1d45a48e53665abaa784c4` |

## Mechanic

Direct-mapped cache misses: mark % c chooses the slot, mark // c is what stays.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
