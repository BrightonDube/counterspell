#!/usr/bin/env bash
# Behaviour tests for the secret and prompt scan (PA19; PIP-33, PIP-41, PIP-32).
#
# The fixtures are built at run time, so this file never contains a token-shaped string.
# The real folders are scanned read-only.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$script_dir/../.."
scanner="$repo_root/scripts/pack-secret-scan.sh"

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# Runs the scanner on one folder. Sets $status, $out and $err.
scan() {
    status=0
    out="$("$scanner" "$1" 2>"$tmp/scan.err")" || status=$?
    err="$(cat "$tmp/scan.err")"
}

# Case 1: the real prompts, skills, agents, summaries and pack scripts contain no secret-like text.
status=0
(cd "$repo_root" && "$scanner" .kiro/pipeline .kiro/agents .kiro/skills/grimoire-pack-authoring \
    .kiro/skills/pack-review docs/pack-reviews scripts/pack-assemble.sh scripts/pack-review-digest.sh \
    scripts/pack-publish-check.sh scripts/kiro-hooks/pack-publish-guard.sh) >/dev/null 2>&1 || status=$?
if [[ "$status" -eq 0 ]]; then pass; else fail "the real pipeline files failed the scan: $status"; fi

# Case 2: a GitHub-style token is refused with SEC-GH. The output names the code and the file, not the token.
fake_token="ghp_"'FAKE0000FAKE0000FAKE0000'
mkdir -p "$tmp/token"
printf 'token = "%s"\n' "$fake_token" > "$tmp/token/notes.md"
scan "$tmp/token"
if [[ "$status" -eq 3 && "$err" == *"SEC-GH"* && "$err$out" != *"$fake_token"* ]]; then
    pass
else
    fail "a token was not refused cleanly: status $status, err '$err'"
fi

# Case 3: a password assignment is refused with SEC-PW.
mkdir -p "$tmp/password"
printf 'password = "hunter2"\n' > "$tmp/password/setup.md"
scan "$tmp/password"
if [[ "$status" -eq 3 && "$err" == *"SEC-PW"* ]]; then pass; else fail "a password was not refused: status $status"; fi

# Case 4: a path to a .env file is refused with SEC-ENV.
mkdir -p "$tmp/envpath"
printf 'Load values from ./.env before running.\n' > "$tmp/envpath/steps.md"
scan "$tmp/envpath"
if [[ "$status" -eq 3 && "$err" == *"SEC-ENV"* ]]; then pass; else fail "a .env path was not refused: status $status"; fi

# Case 5: a retired-value marker is refused with RET-01.
mkdir -p "$tmp/retired"
printf 'See RETIRED-0042 for the old value.\n' > "$tmp/retired/notes.md"
scan "$tmp/retired"
if [[ "$status" -eq 3 && "$err" == *"RET-01"* ]]; then pass; else fail "a retired marker was not refused: status $status"; fi

# Case 6: a script that echoes the credential variable is refused with SEC-ECHO.
mkdir -p "$tmp/echo"
printf 'echo "$COUNTERSPELL_PACKS_PRIVATE_DEPLOY_KEY"\n' > "$tmp/echo/pack-batch.sh"
scan "$tmp/echo"
if [[ "$status" -eq 3 && "$err" == *"SEC-ECHO"* ]]; then pass; else fail "an echoed credential was not refused: status $status"; fi

# Case 7: ordinary text with the words "token" and "password" in a sentence passes.
mkdir -p "$tmp/clean"
printf 'Never print a token or a password. Keep the deploy key private.\n' > "$tmp/clean/rules.md"
scan "$tmp/clean"
if [[ "$status" -eq 0 ]]; then pass; else fail "ordinary text was refused: status $status, err '$err'"; fi

printf 'pack-prompt-secret-scan tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
