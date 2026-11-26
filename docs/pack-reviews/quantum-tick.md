# Pack review summary: quantum-tick

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `quantum-tick` |
| Title | The Quantum Tick |
| Difficulty | hard |
| Track | `matrices` |
| Entry | `quantum_tick` |
| Content hash | `16b1dce624c1f4b5b5765a77e8799c3761ae336ee367e16ab77bbce540de33cb` |

## Mechanic

Round-robin completion-time total — arrivals during a slice queue ahead of the spender's return.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
