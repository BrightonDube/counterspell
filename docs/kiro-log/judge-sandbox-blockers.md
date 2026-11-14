# judge-sandbox blockers

Items here stay open until the named person acts. Kiro left each task unchecked and carried on with work that does not depend on it.

## T02 host spike (HUMAN, SUPERVISED)
- **Needs:** someone on the sandbox VPS to run the steps in `docs/kiro-log/judge-sandbox-human-steps.md` and record the results in `deploy/sandbox-verification.md`.
- **Unblocks:** T03, T21 (file delivery choice), T22 (preflight fixtures), T19 (daemon info fixtures).
- **Status:** not started. Kiro cannot reach the VPS.

## T03 Python digest pin (HUMAN)
- **Needs:** pulling `python:3.13-slim` on the sandbox daemon and writing the real digest into `languages/languages.toml`.
- **Current state:** `languages/languages.toml` holds an all-zero placeholder digest for every language. It is syntactically valid and is never pulled. The sandbox fails closed on a missing image.
- **Unblocks:** running any real container.

## Python driver file (owned by language-drivers)
- **Issue:** `languages/languages.toml` enables Python, and the loader refuses an enabled language whose `drivers/python/driver.py` is missing. So the real catalogue cannot be loaded until language-drivers adds that file. The test suite uses a stand-in driver in a temporary directory.
- **Effect:** the judge, once it loads the catalogue (T24), stops at start-up with `MissingDriver`. This is the intended fail-closed behaviour.
- **Unblocks:** T24 wiring on a real host.

## Design open item
- The design says stderr over 64 KiB kills the run (`docs/system-design.md` section 7 and VAL-5). Amendment A2 changed this to truncation. Brighton must decide whether to update the brief documents. Recorded in `design.md` follow-ups, not edited here.

## Follow-up for run-lifecycle T22 (note, not a blocker)
- `crates/judge/tests/data/source_vectors.json` is the shared validation vector file (judge-sandbox T12). Run-lifecycle T22 tests should load the same file. This was recorded here, and the run-lifecycle spec was not edited.

## Run stop point (14 Nov 2026)
- **Done:** T01, T04 to T13 (with the wave 1 review fixes).
- **Not started in this run:** T14 to T18 (backend traits, fake backend, session lifecycle, call runner, run orchestration, fault properties). They do not need a Docker daemon, but the backend trait shape is a design decision that the next run should review. T15 and T16 are SUPERVISED.
- **Blocked:** T19 to T31 depend on T02 (HUMAN, VPS host spike). T23 and T24 also depend on run-lifecycle T16 and T19. T25 to T31 need a real sandbox daemon and are `sbx_` scenario tests.
- **Human-only:** T02 and T03 (`docs/kiro-log/judge-sandbox-human-steps.md`).

## Run stop point (attempt 2, 14 Nov 2026, after commit 293e4b5)
- **Done this run:** T14 (backend traits, fake), T15 (session lifecycle, cleanup guard), T16 (call runner), T17 (run orchestration), T18 (fault-injection properties), and the wave 2 review fixes. Each has its own commit (T14 `7ecee7c`, T15 `9b85496`, T16 `0a5094b`, T17 `98b34db`, T18 `c67110d`, fixes `293e4b5`).
- **Blocked, unchanged:** T19 to T31 depend on T02 (HUMAN, VPS spike). T19 needs the recorded daemon fixtures from T02. T20 and T21 need T19. T23 and T24 need T20 and run-lifecycle T16 and T19. T25 to T31 need a sandbox daemon.
- **Human-only, not run:** T02 and T03. Steps are in `docs/kiro-log/judge-sandbox-human-steps.md`.
- **New dependency to watch:** the sweeper (T23) is what makes a failed removal safe. Until T23 lands, a removal failure leaves a container behind. See the wave 2 entry in `judge-sandbox-review-queue.md`.
- **Contracts the bollard backend must meet (written in `backend.rs`):** every daemon call times out at `SANDBOX_API_TIMEOUT`; `Exited` only after a successful inspect; exec output capped at 64 KiB with `overflowed` set.
- **Residual risk, not fixed:** a program can answer a later call before that call's input is written, if its answer matches. See the review queue.
