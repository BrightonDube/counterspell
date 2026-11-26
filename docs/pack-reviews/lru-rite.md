# Pack review summary: lru-rite

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `lru-rite` |
| Title | The Lru Rite |
| Difficulty | medium |
| Track | `arrays` |
| Entry | `lru_rite` |
| Content hash | `5fabcaa41196a06302acac6d6477ff059f43df9dac590235bf40871e8f6f277c` |

## Mechanic

LRU page faults — a hit renews the page's freshness before the next eviction.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
