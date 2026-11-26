# Pack review summary: depend-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `depend-rite` |
| Title | The Depend Rite |
| Difficulty | medium |
| Track | `databases` |
| Entry | `depend_rite` |
| Content hash | `4375bd46586013623817a80ae5fb563b9a2504fc520a95045c9863cf678d3d26` |

## Mechanic

Functional-dependency check: every repeated determinant must carry one consistent dependent.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
