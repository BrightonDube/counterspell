# Pack review summary: token-scope

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `token-scope` |
| Title | The Token Scope |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `scope_ok` |
| Content hash | `84ccfee4ac0f77b0a151a67ed0b17f3a52274f031d6b6345de5b00ca3fd0d1b1` |

## Mechanic

Scoped-token check — expiry plus required-scope coverage.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
