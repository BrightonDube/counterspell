# Pack review summary: burst-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `burst-rite` |
| Title | The Burst Rite |
| Difficulty | easy |
| Track | `matrices` |
| Entry | `burst_rite` |
| Content hash | `97169a32fdf67bd9144dc9f9dc99a0856a0cd3d0c67ee360d7d21478787ec118` |

## Mechanic

First-come-first-served waiting-time total over an engine that may idle between arrivals.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
