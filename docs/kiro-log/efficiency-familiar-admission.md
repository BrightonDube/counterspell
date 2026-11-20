# efficiency-familiar T40 — bench admission log

Run on the local rootless sandbox (`SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock`,
cgroup v2, seccomp builtin), catalogue `postgres://127.0.0.1:55432/counterspell`.
Method per pack: `validate --public` clean → `import --sandbox` (reference through the
real runner) → `publish` → `counterspell-judge baseline --problem <slug> --lang python`.

## Results

| Pack | Declared class | Bench | Admission | Stored |
|------|----------------|-------|-----------|--------|
| count-pairs | `n^2` (nested-loop reference) | `values`: 128–4096 | **admitted** | 4 baseline rows (python) |
| longest-run-length | — | — | refused | — |
| diagonal-total | — | — | refused | — |

## Per-pack detail

- **count-pairs** (`n^2`, sizes `[128, 512, 2048, 4096]`): fit confirmed Quadratic at
  ≥Medium; `stored 4 new baseline rows`. First attempt with a 16384 size exceeded the
  4 s submit-bench budget; 8192 was dropped for the same reason — final worst size
  4096 ≈ 8.4 M pair-checks fits comfortably.

- **longest-run-length**: reference is a single O(n) pass, but the fit never confirmed
  Linear. Across `[256…65536]` (256× span), `[1024, 8192, 65536]` and
  `[4096, 16384, 65536]` the report stayed `not confirmed` with
  `plausible [Linear, Linearithmic]` — at ≤65 536 characters the measured Python loop
  is too cheap for the timing noise to separate n from n·log n. Bench **not** enabled
  (declaration removed; admission requires ≥Medium, R13.2/R13.3).

- **diagonal-total**: the only honest class was `n` (measured time is dominated by
  parsing the generated grid — linear in the cell count — while the diagonal walk is
  O(√n)). Fit stayed `not confirmed` (`plausible [Linear, Linearithmic]`) across
  `[64…16384]` and `[256, 1024, 4096, 16384, 65536]`. Bench **not** enabled.

- **JavaScript baselines**: not taken — the private tree ships no JavaScript
  references for these packs, so `--lang javascript` has no fixture to time. Recorded
  honestly rather than synthesizing references.

## Shortfall (spec target: ≥3)

Only **1 of 3** launch problems passed admission. The two linear-sub-ms references
cannot reach `Medium` confidence inside the submit-bench budget on this host — the
fit cannot separate Linear from Linearithmic. This is the honest result; the
criteria were not loosened. Flagged for Brighton: enabling bench on cheap linear
references likely needs either larger allowed sizes (>65 536), repeat-count control
in `baseline`, or a deliberately heavier reference implementation.
