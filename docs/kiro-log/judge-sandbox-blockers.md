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
