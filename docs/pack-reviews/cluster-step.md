# Pack review summary: cluster-step

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `cluster-step` |
| Title | The Cluster Step |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `cluster_step` |
| Content hash | `9983c3ed4708b1e318abd1247bcdb18b05a4c5555043f3dd515b8e086731ac2e` |

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
