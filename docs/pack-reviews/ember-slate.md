# Pack review summary: ember-slate

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `ember-slate` |
| Title | The Ember Slate |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `slate_len` |
| Content hash | `b73a43d5a465a5ebadd51b49af4a14c2e8496961d7563b5218a3c3dfe7bf23e9` |

## Mechanic

Longest strictly climbing pick — O(n²) per-position best.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
