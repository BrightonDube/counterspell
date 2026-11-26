# Pack review summary: cluster-step

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `cluster-step` |
| Title | The Cluster Step |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `cluster_step` |
| Content hash | `b7fe74248ea643a184f3515552467258de16d106ee0497f955e87d6c5238391c` |

## Mechanic

Nearest-centroid churn — argmin with low-tie rule versus prior labels.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
