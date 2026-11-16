#!/usr/bin/env bash
# Behaviour tests for the publish guard and the pack-related shell and validate checks (PA16;
# PIP-1, PIP-28, PIP-38, PIP-43). Each case feeds a hook event on stdin and checks the exit code.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$script_dir/../.."
publish_guard="$repo_root/scripts/kiro-hooks/pack-publish-guard.sh"
dangerous_shell="$repo_root/scripts/kiro-hooks/dangerous-shell.sh"
pack_validate="$repo_root/scripts/kiro-hooks/pack-validate.sh"

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# Runs a hook script on the given event text. Sets $status.
run_hook() {
    local script="$1" event="$2"
    status=0
    printf '%s' "$event" | bash "$script" >/dev/null 2>&1 || status=$?
}

expect() {
    local name="$1" want="$2" script="$3" event="$4"
    run_hook "$script" "$event"
    if [[ "$status" -eq "$want" ]]; then pass; else fail "$name: want exit $want, got $status"; fi
}

# A write that sets status = "published" in a pack's meta.toml is blocked.
expect "published write is blocked" 2 "$publish_guard" \
    '{"tool":"fs_write","path":"packs/count-pairs/meta.toml","content":"status = \"published\"\n"}'

# A str_replace that sets published in meta.toml is blocked too.
expect "published str_replace is blocked" 2 "$publish_guard" \
    '{"tool":"str_replace","path":"packs/count-pairs/meta.toml","oldStr":"draft","newStr":"status = \"published\""}'

# Any write to approval.toml is blocked, wherever it sits.
expect "approval.toml write is blocked" 2 "$publish_guard" \
    '{"tool":"fs_write","path":"private/packs/count-pairs/approval.toml","content":"slug = \"count-pairs\"\n"}'

# An ordinary draft edit is allowed.
expect "draft meta edit is allowed" 0 "$publish_guard" \
    '{"tool":"fs_write","path":"packs/count-pairs/meta.toml","content":"status = \"draft\"\n"}'

# A statement that mentions the word published is not a metadata write.
expect "problem.md mentioning published is allowed" 0 "$publish_guard" \
    '{"tool":"str_replace","path":"packs/count-pairs/problem.md","oldStr":"x","newStr":"status = \"published\""}'

# A document that only talks about approval.toml is allowed, because only the path matters.
expect "docs that mention approval.toml are allowed" 0 "$publish_guard" \
    '{"tool":"fs_write","path":"docs/kiro-log/notes.md","content":"approval.toml is written by Brighton."}'

# Running a pack file through python3, node or a shell is blocked.
expect "python3 on a pack file is blocked" 2 "$dangerous_shell" \
    '{"tool":"execute_bash","command":"python3 packs/count-pairs/reference/solution.py"}'
expect "node on an assembled file is blocked" 2 "$dangerous_shell" \
    '{"tool":"execute_bash","command":"node target/pack-assembly/count-pairs/incorrect/a.js"}'

# Ordinary shell commands are allowed.
expect "cargo test is allowed" 0 "$dangerous_shell" \
    '{"tool":"execute_bash","command":"cargo test -p counterspell-grimoire"}'

# A private file name under packs/ is refused by the validate hook before any check runs.
status=0
bash "$pack_validate" "packs/count-pairs/hidden.toml" >/dev/null 2>&1 </dev/null || status=$?
if [[ "$status" -eq 2 ]]; then pass; else fail "hidden.toml under packs/ was not refused: $status"; fi

printf 'pack-publish-guard tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
