# Hook testing notes

These checks are documented for a later IDE or CLI V3 run. They were **not executed in this non-interactive run**.

Run the scripts from the repository root and provide a JSON event on standard input. The hook definitions invoke the scripts with `bash`, so executable-bit changes are not required.

## `guard-secrets`

A write targeting `.env` must block with exit status `2`:

```bash
printf '%s\n' '{"path":".env","content":"DATABASE_URL=hidden"}' \
  | bash scripts/kiro-hooks/guard-secrets.sh write
printf 'status=%s\n' "$?"
```

Expected result: the script prints a `blocked:` message and `status=2`. In the IDE, the `PreToolUse` hook must prevent the write.

A write targeting `.env.example` must be allowed with exit status `0`:

```bash
printf '%s\n' '{"path":".env.example","content":"DATABASE_URL=postgres://localhost/dev"}' \
  | bash scripts/kiro-hooks/guard-secrets.sh write
printf 'status=%s\n' "$?"
```

Expected result: no block and `status=0`. Also check a representative PEM path, `id_rsa`, `secrets/config.toml`, and the API-key patterns from the playbook.

## `guard-dangerous-shell`

A shell event containing `docker run --privileged` must block with exit status `2`:

```bash
printf '%s\n' '{"command":"docker run --privileged counterspell-test"}' \
  | bash scripts/kiro-hooks/dangerous-shell.sh
printf 'status=%s\n' "$?"
```

Expected result: the script prints `blocked: privileged containers are not allowed` and `status=2`. Repeat for host networking, unconfined seccomp, a Docker socket mount, `rm -rf /`, `git push --force`, and `curl ... | sh`.

An ordinary read-only shell command must return `0` and print nothing.

## Failure status through `tail`

When the workspace contains `crates/engine`, run the same shape used by the hook and inspect the status immediately, without allowing `tail` to replace it:

```bash
set -o pipefail
cargo test -q -p counterspell-engine 2>&1 | tail -40
status=$?
printf 'status=%s\n' "$status"
```

Use a deliberately failing engine test. The status must be nonzero. The `engine-tests.sh`, `rust-check.sh`, and `web-check.sh` scripts use Bash `pipefail` and preserve that status. This is the regression check for the earlier `cargo test | tail` masking bug.

## Hook-level checks

After the direct script checks, exercise the actual IDE/CLI hook events with a temporary branch and disposable files. Confirm that the two blocking guards stop their operations, while `pre-task-green` and `ghost-hunt-on-test-failure` always allow the operation after printing their information. Do not place real credentials in any fixture.
