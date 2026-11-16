# judge-sandbox: done, pending tick

Each entry: task id, commit(s), tests that prove it, the exact verification commands and their result. A task is done only after Kiro re-verifies and ticks it.

## T20 bollard backend: create, read-back, image check (SUPERVISED)

- Commits: `d74705c` (backend part 1 + model tests), `149a08f` (sbx test), `a16d2ad` (read-back tolerates image-declared env).
- Proving tests: `tests/model_bollard.rs` (10 tests incl. 26-row mutation table), `tests/sbx_bollard_create.rs::sbx_create_readback_matches` on the rootless daemon.
- Verification run 16 Nov 2026:
  - `cargo test -p counterspell-judge --features sandbox --test sbx_bollard_create -- --ignored --nocapture` with `SANDBOX_DOCKER_SOCKET` from `~/.config/counterspell/dev-test.env`: **PASS** (1 passed, 0.22 s). Identity check passed (cpuset/io delegated), shipped catalogue loaded now that `languages/drivers/python/driver.py` and `languages/drivers/javascript/driver.js` exist, create + read-back matched the policy, container removed.
  - Workspace gate: `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok; `cargo test --workspace` 1369 passed, 0 failed.
