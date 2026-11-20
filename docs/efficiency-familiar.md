# The Efficiency Familiar

The Familiar is the small creature that reacts to how a submission *scales*, not
just whether it passes. It is deliberately conservative: it would rather say
nothing than say something wrong.

## What it actually measures

On an accepted run, the judge re-executes the submitted program inside the same
rootless sandbox on a series of generated inputs of increasing size and records
median wall-clock times. That point set goes through the `fit-1` estimator
(`crates/engine/src/complexity/`), which compares the observed growth against a
fixed family of complexity classes — `1`, `log n`, `n`, `n log n`, `n^2`,
`n^3`, `2^n` — and reports:

- **observed class** — the single best-fitting class, only when the fit is
  confirmed;
- **confidence** — `low`, `medium` or `high`. Low confidence means the
  measurement happened but no claim is made;
- **leaning / plausible classes** — what the data suggests when several classes
  still fit;
- **ratio vs baseline** — median time of the submission divided by the stored
  reference baseline at the largest admitted size.

A failed verdict, an expired run, a host that was too busy, or a bench that did
not finish all sizes produces **no measurement at all** — never a guess.

## The five moods

`decide()` in `crates/common/src/familiar/mood.rs` is a pure function over the
measured facts. Moods, worst to best:

| Mood | Value | When |
|------|-------|------|
| FURIOUS | 1 | Any non-Accepted verdict, or a confirmed class three or more tiers worse than the reference |
| ANGRY | 2 | Confirmed class two tiers worse than the reference |
| UNEASY | 3 | Confirmed class one tier worse than the reference |
| PLEASED | 4 | Accepted but unmeasured (no bench spec, host busy, bench failed/incomplete), low confidence, no stored baseline, or same class but slower constant factor (ratio > 2.0) |
| LAUGHING | 5 | Confirmed class matches the reference, or is better |

Mood 5 on a submit also triggers the laugh-burst animation; everything else is
effect-free. Speech, when enabled, is AI-labelled flavour text and **never
influences the verdict or the mood**.

## Honest limits

- The Familiar estimates *growth with input size* on a bounded sample. It does
  not prove a program's asymptotic complexity and it never claims a solution is
  the best possible.
- Medium/high confidence needs enough points and one clear winner; otherwise
  the fit is reported as low confidence and the mood stays at PLEASED.
- `host_busy`, `bench_incomplete` and `low_confidence` are not failures — they
  mean the Familiar declined to judge rather than risk a wrong claim.
- Moods compare against the pack's stored reference class. If a pack's
  reference is itself quadratic, an honest quadratic submission legitimately
  earns LAUGHING (`matches_reference`); that is the designed behaviour, not a
  bug.
- A non-Accepted run never produces a complexity claim, and an incomplete run
  produces no mood or reaction.

## Baselines

Baselines are per-pack, per-language measured medians stored by the trusted
judge, not by pack code. Regenerate them on a host with the rootless sandbox:

```bash
DATABASE_URL=... SANDBOX_DOCKER_SOCKET=... \
  cargo run -p counterspell-judge -- baseline <slug>
```

Admission is gated: a baseline is stored only when the reference passes its own
fit at medium confidence or better. Packs whose references cannot be confirmed
(the fit stays ambiguous at the allowed sizes) keep bench disabled; the
admission log `docs/kiro-log/efficiency-familiar-admission.md` records the
per-pack outcome.

## Constants

| Constant | Value | Meaning |
|----------|-------|---------|
| `FIT_VERSION` | `fit-1` | Estimator version stored with every fit and baseline; any change to the estimator bumps it |
| `SUBMIT_BENCH_BUDGET` | 4 s | Total wall-clock budget for a submit-mode bench series |
| `RUN_BENCH_BUDGET` | 1500 ms | Budget for a run-mode bench series |

## Licences and notices

The Familiar's animation uses GSAP 3.15.1 under the GreenSock "Standard No
Charge" licence — an accepted exception recorded in `docs/licence-rules.md`.
The source is not vendored; see `THIRD-PARTY-NOTICES.md`.
