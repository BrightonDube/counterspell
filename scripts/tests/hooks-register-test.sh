#!/usr/bin/env bash
# Checks the pack hooks as Kiro will load them (PA18; PIP-43). Reads the JSON with jq and
# tests each matcher against example tool names and paths. Nothing runs a hook here.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$script_dir/../.."
publish_hook="$repo_root/.kiro/hooks/pack-publish-guard.json"
validate_hook="$repo_root/.kiro/hooks/pack-validate.json"

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# Returns 0 when the regex in $1 matches the text in $2 (both are plain strings).
matches() {
    [[ "$2" =~ $1 ]]
}

# 1. Both files parse as JSON.
if jq -e . "$publish_hook" >/dev/null 2>&1 && jq -e . "$validate_hook" >/dev/null 2>&1; then
    pass
else
    fail "a hook file is not valid JSON"
fi

# 2. The publish guard runs the script that exists in the repository.
publish_command="$(jq -r '.hooks[0].action.command' "$publish_hook")"
publish_script="${publish_command#bash }"
if [[ "$publish_command" == bash\ * && -f "$repo_root/$publish_script" ]]; then
    pass
else
    fail "the publish guard command does not point at an existing script: '$publish_command'"
fi

# 3. The publish guard matches agent writes and str_replace edits.
publish_matcher="$(jq -r '.hooks[0].matcher' "$publish_hook")"
if matches "$publish_matcher" "fs_write" && matches "$publish_matcher" "str_replace"; then
    pass
else
    fail "the publish guard matcher does not cover fs_write and str_replace"
fi

# 4. The publish guard is a PreToolUse hook, so it can block the write before it happens.
if [[ "$(jq -r '.hooks[0].trigger' "$publish_hook")" == "PreToolUse" ]]; then
    pass
else
    fail "the publish guard is not a PreToolUse hook"
fi

# 5. The validate hook matches saves under packs/ and under the assembled tree.
validate_matcher="$(jq -r '.hooks[0].matcher' "$validate_hook")"
if matches "$validate_matcher" "packs/count-pairs/meta.toml" \
    && matches "$validate_matcher" "target/pack-assembly/count-pairs/hints.toml"; then
    pass
else
    fail "the validate matcher misses packs/ or the assembled tree"
fi

# 6. The validate hook does not match ordinary source files.
if ! matches "$validate_matcher" "crates/grimoire/src/lib.rs"; then
    pass
else
    fail "the validate matcher also matches ordinary source files"
fi

printf 'hooks-register tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
