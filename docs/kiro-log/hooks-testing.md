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

## `guard-test-integrity`

Pipe a sample `PreToolUse` event into `bash scripts/kiro-hooks/guard-tests.sh` from a scratch directory that contains `docs/requirements.md` and `docs/kiro-log/`. Event shape: `{"tool_name":"str_replace","tool_input":{"path":"src/x.rs","oldStr":"...","newStr":"..."}}`. The CLI spellings `old_str`, `new_str` and `file_text` are accepted too.

Must block with status `2`: an edit that leaves fewer `#[test]` / `#[tokio::test]` / `def test_` / `it(` / `test(` markers than it started with; overwriting a file with fewer tests; `delete_file` on a file that contains tests; adding `#[ignore]` to a test not named `pg_*` or `valkey_*`; adding `@pytest.mark.skip`, `it.skip` or `xit`; a `test-change:` marker whose ID is not in `docs/requirements.md`.

Must allow with status `0`: adding, renaming, moving or refactoring tests (same or higher count); `#[ignore]` on `pg_*` / `valkey_*` tests; non-source files such as `.md`; creating a new file; appending; deleting a file without tests; malformed or empty events; MCP tools with no path; and `// test-change: VAL-3` with a real ID (which also appends a line to `docs/kiro-log/test-guard.log`).

Checked 12 Nov 2026 against the script with 25 such cases in a scratch directory: 25 passed. Not yet checked against a live IDE or CLI hook event, so confirm the matcher `write|str_replace|fs_append|delete_file` fires in a real session.


## Token-boundary API-key patterns (13 Nov 2026)

The write-mode check for `sk-`, `gsk_` and `re_` keys now requires a token boundary (`(^|[^A-Za-z0-9_])` before the prefix). Without that, ordinary Rust identifiers such as `failure_reason_public_messages_are_distinct` matched `re_[A-Za-z0-9_-]{20,}` (case-insensitive) and the PreToolUse hook blocked every test write in the run-lifecycle session. The hook also prints which pattern class matched.

Regression checks (no full key is stored in the test file; prefixes and bodies are concatenated at runtime):

```bash
bash scripts/kiro-hooks/test-guard-secrets.sh
```

Expected result: every case is `ok`, including allowing `failure_reason_public_messages_are_distinct` and a long `ask-` word, and still blocking `.env`, PEM/`id_rsa`/`secrets/` paths, and real `sk-` / `gsk_` / `re_` keys. Checked 13 Nov 2026: all cases passed.
