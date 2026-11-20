---
inclusion: always
---
# One workspace, one build (disk and memory hygiene)

Brighton's disk filled and WSL crashed because agents made extra clones, git worktrees and Rust `target/` directories. One Rust build tree is often 5 to 15 GB, and a cold build of this workspace needs several GB of RAM. These rules apply to every agent, workflow and script in this repository. They override any global default that says otherwise.

## One workspace, one agent
- Work only in this checkout. Do not run `git worktree add`, `git clone` of this repository, `cp -r` of it, or `git init` for a scratch copy of its history.
- Workflows run in place. Never ask for a worktree or a `.worktrees/` folder, even if a default says to.
- If a task truly needs a scratch copy (for example a history rewrite), stop and ask Brighton first. If he agrees, delete the copy before you end your turn and say so in your summary.
- Run one agent or workflow at a time. Before you start, check `pgrep -fa 'kiro-cli|spec-driver|kiro-implement|cargo|rustc'`. If something else is running, stop and tell Brighton. Do not start parallel workflow steps that edit or build this repository.
- Remove anything you create outside the repository (scratch folders, bundles, logs) before you finish. The one exception is `~/counterspell-backups/`, which Brighton clears himself.

## One Rust build tree
- The only build directory is `./target` at the repository root. Never set `CARGO_TARGET_DIR`, `--target-dir` or `build.target-dir`. Never build from a copy or a second checkout.
- Run one cargo command at a time. Use `CARGO_BUILD_JOBS=4` for clippy and test runs. Each `rustc` can use about 400 MB, and WSL is capped at 8 GB.
- Use one flag set for gate commands, the same as `scripts/pre-push.sh`: `--workspace --all-targets --all-features --locked`. Mixing plain and `--all-features` runs creates two sets of artifacts.
- Do not build `--release` or for another target unless the task says so.

## Keep it small
- Before a full gate, run `df -h .` and `du -sh target`. If less than 20 GB is free, stop and tell Brighton.
- If `target/` is over 15 GB, shrink it with `cargo sweep --maxsize 12GiB` when `cargo-sweep` is installed (`cargo sweep --version`). Otherwise run `cargo clean`.
- If a build dies from a crash, out-of-memory, a full disk, a killed process or a corrupt-artifact error, run `cargo clean` once and rebuild once. This is not for ordinary compile errors or failing tests. A clean rebuild takes about 10 minutes here and raises the memory peak, so do not use it as a routine step.
- Never run `cargo clean` while another cargo or test process is running.

## Why
Cargo has no built-in limit on how many build artifacts pile up. Separate target directories per worktree multiply that growth, and a shared directory between worktrees risks wrong or corrupt artifacts (see rust-lang/cargo#16804). So the rule is one tree, size-capped, never copied.
