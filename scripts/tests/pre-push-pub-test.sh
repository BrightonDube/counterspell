#!/usr/bin/env bash
# Behaviour tests for the publish check used by scripts/pre-push.sh (PA17; PIP-28, PIP-44; PUB-01).
#
# Each case builds a throwaway git repository in a temporary folder with two commits and runs the
# check over that range. Nothing is pushed, nothing is configured, and no global git setting changes.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
check="$script_dir/../pack-publish-check.sh"

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# Commits a file in the repository at $1 with the message $2. Identity is passed per command.
commit_file() {
    local repo="$1" content="$2" message="$3"
    mkdir -p "$repo/packs/count-pairs"
    printf '%s' "$content" > "$repo/packs/count-pairs/meta.toml"
    git -C "$repo" add packs/count-pairs/meta.toml
    git -C "$repo" -c user.name=test -c user.email=test@example.invalid commit -q -m "$message"
}

# Builds a repository whose first commit is a draft and whose second commit publishes the pack.
# The second commit's message is $1.
build_repo() {
    local name="$1" message="$2"
    local repo="$tmp/$name"
    git init -q "$repo"
    commit_file "$repo" 'slug = "count-pairs"
status = "draft"
' "draft count-pairs"
    commit_file "$repo" 'slug = "count-pairs"
status = "published"
' "$message"
    printf '%s' "$repo"
}

# Runs the check over the last commit of $1. Sets $status and $err.
run_check() {
    local repo="$1"
    status=0
    err="$(cd "$repo" && "$check" HEAD~1..HEAD 2>&1 >/dev/null)" || status=$?
}

# Case 1: publishing without the approval phrase is refused with PUB-01.
repo="$(build_repo unapproved "publish count-pairs")"
run_check "$repo"
if [[ "$status" -eq 1 && "$err" == *"PUB-01"* ]]; then pass; else fail "unapproved publish: status $status, err '$err'"; fi

# Case 2: the same publish with the approval phrase is allowed.
repo="$(build_repo approved "publish count-pairs

Approved by Brighton")"
run_check "$repo"
if [[ "$status" -eq 0 ]]; then pass; else fail "approved publish was refused: status $status, err '$err'"; fi

# Case 3: a commit that keeps the pack a draft needs no phrase.
repo="$tmp/draft-only"
git init -q "$repo"
commit_file "$repo" 'slug = "count-pairs"
status = "draft"
' "draft count-pairs"
commit_file "$repo" 'slug = "count-pairs"
status = "draft"
notes = "edited"
' "edit the draft"
run_check "$repo"
if [[ "$status" -eq 0 ]]; then pass; else fail "a draft edit was refused: status $status"; fi

# Case 4: the phrase in another commit does not approve a publish made in a later commit.
repo="$tmp/phrase-elsewhere"
git init -q "$repo"
commit_file "$repo" 'slug = "count-pairs"
status = "draft"
' "draft count-pairs"
commit_file "$repo" 'slug = "count-pairs"
status = "draft"
notes = "reviewed"
' "Approved by Brighton"
commit_file "$repo" 'slug = "count-pairs"
status = "published"
notes = "reviewed"
' "publish count-pairs"
status=0
err="$(cd "$repo" && "$check" HEAD~2..HEAD 2>&1 >/dev/null)" || status=$?
if [[ "$status" -eq 1 && "$err" == *"PUB-01"* ]]; then pass; else fail "the phrase in another commit was accepted"; fi

printf 'pre-push-pub tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
