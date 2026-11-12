# Implementation order (all specs written 12 Nov 2026; implementation approved by Brighton 12 Nov 2026, 16:42 SAST)

Runner: `scripts/kiro-implement-all.sh` (tmux session `kiro-impl`, progress in `/tmp/kiro-impl.progress`). One spec at a time, strictly in this order. `repo-skeleton` is already done (28/28).

| # | Spec | Prompt | Model | Tasks (core / stretch) | Gate | Why here |
|---|---|---|---|---|---|---|
| 1 | ghost-engine | 20-ghost-engine-implement-cli.md | gpt-5.6-luna | 37 / 0 | none | Pure Rust engine with no dependencies. Grimoire T07 and T17 are blocked until its public API (`CandidateStream`, `GeneratorConfig`, `Oracle`, `ProgramResult`) exists. Heart of the core loop. |
| 2 | grimoire-packs | 21-grimoire-packs-implement-cli.md | gpt-5.6-luna | 30 / 0 | none | The problems. Needs the engine API; one task needs judge-sandbox and is logged as a blocker. |
| 3 | run-lifecycle | 22-run-lifecycle-implement-cli.md | gpt-5.6-luna | 56 / 0 | none | Postgres queue, claim/fence, SSE replay. Uses `UserResolver`/`ProblemLookup` seams and a judge `testkit` feature, so it needs neither auth nor Docker. judge-sandbox T24 needs its T16 and T19. |
| 4 | judge-sandbox | 23-judge-sandbox-implement-cli.md | claude-sonnet-5.5 | 56 / 0 | **T00: Brighton's boundary review** (a "Review decisions" note in its design.md, or T00 ticked) | Security boundary. Its own tasks.md says no task may start before T00. The runner skips it until that note exists. Sonnet because it is the most security-critical code. |
| 5 | language-drivers | 24-language-drivers-implement-cli.md | gpt-5.6-luna | 22 / 5 | judge-sandbox at least half done | T01 needs the judge-sandbox interface, and it extends `languages.rs` from judge-sandbox T04. |
| 6 | auth | 25-auth-implement-cli.md | gpt-5.6-luna | 55 / 0 | none | Sessions, OAuth, magic link, CSRF, and the deferred `submissions.user_id` FK (needs run-lifecycle's table). Its 20 `[SUPERVISED]` tasks go to Brighton's review queue. Luna, not sonnet, so the budget can still cover judge-sandbox (see below). |
| 7 | ide-workspace | 26-ide-workspace-implement-cli.md | gpt-5.6-luna | 39 / 0 | none | Needs run-lifecycle events and auth cookies. Browser-only tasks deferred (no E2E yet). |
| 8 | efficiency-familiar | 27-efficiency-familiar-implement-cli.md | gpt-5.6-luna | 60 / 4 | none | The mascot. Measured layer first; judge-side measurement tasks that need the sandbox are logged as blockers. |
| 9 | hints | 28-hints-implement-cli.md | gpt-5.6-luna | 29 / 0 | none | Needs auth and runs; independent of the sandbox. |
| 10 | dashboard-fsrs | 29-dashboard-fsrs-implement-cli.md | gpt-5.6-luna | 27 / 9 | none | Needs auth and submissions. FSRS stretch is skipped this pass. |
| 11 | counterspell-mcp-power | 30-counterspell-mcp-power-implement-cli.md | gpt-5.6-luna | 31 / 0 | judge-sandbox at least half done | Its T02 needs the completed engine, Grimoire and judge-sandbox contracts. |
| 12 | landing-steel-night | 31-landing-steel-night-implement-cli.md | gpt-5.6-luna | 23 / 0 | none | `web/` only; extends `web/src/lib/` files from ide-workspace, so it comes after it. |
| 13 | break-room | 32-break-room-implement-cli.md | gpt-5.6-luna | 21 / 0 | **T00: freeze-safe core verified on the VPS** (env `BREAK_ROOM_OK=1`) | First feature cut by design. T00 needs the real fail-to-ghost flow running on the VPS, which a local run cannot verify. |

Rules for every spec are in each prompt and in `.kiro/steering/tdd-and-quality.md`. `[stretch]` tasks are skipped this pass (credit budget and the 2 Dec freeze). `[SUPERVISED]` tasks are implemented test-first and listed in `docs/kiro-log/<spec>-review-queue.md` for Brighton's review before anything merges to `main`. `[HUMAN]` tasks are prepared (commands written down) but left unchecked.

## Credit estimate (12 Nov 2026, ~1,480 left of 4,000; the runner keeps a 250 floor)

Evidence from `docs/KIRO_USAGE.md`: repo-skeleton implementation cost 39.34 credits for 28 tasks on luna (1.4 per task). Spec writing cost about 0.27 credits per KB on sonnet against about 0.10 on luna, so sonnet is about 2.7x luna in practice. Assume TDD, debugging and reviews add 30-70% and these tasks are heavier than the skeleton's: about **2.5 credits per task on luna** and **6.5 on sonnet** (wide error bars, roughly -30%/+40%).

| Spec | Model | Est. credits |
|---|---|---|
| ghost-engine | luna | ~95 |
| grimoire-packs | luna | ~75 |
| run-lifecycle | luna | ~140 |
| judge-sandbox (gated) | sonnet | ~365 |
| language-drivers (gated) | luna | ~55 |
| auth | luna | ~140 |
| ide-workspace | luna | ~100 |
| efficiency-familiar | luna | ~150 |
| hints | luna | ~75 |
| dashboard-fsrs | luna | ~70 |
| counterspell-mcp-power (gated) | luna | ~80 |
| landing-steel-night | luna | ~60 |
| break-room (gated) | luna | ~55 |

**Verdict.** About 1,230 credits are usable (1,480 left minus the 250 floor).
- The nine ungated specs come to about 905, so they should fit.
- All 13 come to about 1,460, so they do not fit. If judge-sandbox runs early (Brighton signs off T00 before the runner reaches #4), the guard will stop later specs instead. In order, landing-steel-night, then dashboard-fsrs, then hints are the likely casualties.
- If everything ran on luna, all 13 would cost about 1,235, right at the limit.
- **Sonnet everywhere is not affordable.** At about 6.5 credits a task, the ungated nine alone would cost about 2,300, and the money would run out during run-lifecycle (the third spec).

Why luna is the default instead of sonnet: luna implemented repo-skeleton 28/28 with every CI check green, at a third of sonnet's real cost. Sonnet is kept for judge-sandbox, the security boundary. To change a model, edit the `SPECS` table in `scripts/kiro-implement-all.sh`.
