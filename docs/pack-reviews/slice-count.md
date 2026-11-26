# Pack review summary: slice-count

## Public metadata

| Field | Value |
| ----- | ----- |
| Slug | `slice-count` |
| Title | The Slice Count |
| Difficulty | easy |
| Track | `arrays` |
| Entry | `slice_count` |
| Content hash | `15751680856c40e9cd2adf36e65ca029fb407aff9bbb984a5543c4ddc00937a6` |

## Mechanic

Ceiling-division slice count — a zero-tick rite needs no slice.

## Review basis

- Statement, metadata, hints, fixtures and originality record reviewed in the public tree.
- Reference, hidden cases, generator and known ghosts reviewed in the private tree.
- Samples and hand-traced hidden cases were cross-checked against the reference before assembly.
- Ghost inputs and recorded outcomes were re-run during verification.

## Sandbox and oracle notes

- S10/S11 verification covers the merged tree through the real PackRunnerFactory and SandboxOracle.
- Hidden, generated and known-ghost cases run inside the rootless sandbox; public summaries carry no hidden values.
