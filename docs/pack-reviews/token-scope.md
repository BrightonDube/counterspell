# Pack review summary: token-scope

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `token-scope` |
| Title | The Token Scope |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `scope_ok` |
| Content hash | `f16940735edbbdb8a2355516c55c74abfd5cfcb3d4628393824adda16c6c24fc` |

## Mechanic

Scope-set containment — checking a token's grants cover the required scope.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
