#!/usr/bin/env bash
# Behaviour tests for scripts/pack-review-digest.sh (PA15; PIP-27, PIP-32).
#
# Temporary directories and synthetic canaries only. Nothing reads the real private repository.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
digest="$script_dir/../pack-review-digest.sh"

readonly CANARY="CANARY-HIDDEN-pa15"

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# A synthetic assembled pack with one hidden file that carries the canary.
assembled="$tmp/assembly"
mkdir -p "$assembled/demo"
printf '# Demo\n' > "$assembled/demo/problem.md"
printf '[[case]]\nexpected = "%s"\n' "$CANARY" > "$assembled/demo/hidden.toml"
reports="$tmp/reports"
mkdir -p "$reports"
printf 'S9 ok\n' > "$reports/s9.txt"

export PACK_REVIEW_ROOT="$tmp/review"
export PACK_REVIEWS_DIR="$tmp/docs/pack-reviews"

# Case 1: prepare writes the reviewer input under the private review root and prints only a path.
status=0
out="$("$digest" prepare demo "$assembled" "$reports" 2>"$tmp/prepare.err")" || status=$?
err="$(cat "$tmp/prepare.err")"
if [[ "$status" -eq 0 && -f "$tmp/review/demo/input/demo/hidden.toml" ]]; then
    pass
else
    fail "prepare did not write the reviewer input: status $status, err '$err'"
fi

# Case 2: prepare never prints the canary, and the input keeps the hidden file in its private folder.
if [[ "$out$err" != *"$CANARY"* ]]; then pass; else fail "prepare printed a canary"; fi

# Case 3: a clean summary is copied into the public summary folder.
clean="$tmp/clean.md"
printf '# Review summary: demo\n\nThe problem asks for a count of pairs.\n' > "$clean"
status=0
"$digest" publish-summary demo "$clean" >/dev/null 2>"$tmp/clean.err" || status=$?
if [[ "$status" -eq 0 && -f "$PACK_REVIEWS_DIR/demo.md" ]]; then
    pass
else
    fail "a clean summary was not published: status $status"
fi

# Case 4: a summary that points at a hidden case is refused with SUM-01 and nothing is written.
pointer="$tmp/pointer.md"
printf 'The ghost is at hidden.toml /case/3 in the tree.\n' > "$pointer"
rm -f "$PACK_REVIEWS_DIR/pointer.md"
status=0
"$digest" publish-summary pointer "$pointer" >/dev/null 2>"$tmp/pointer.err" || status=$?
if [[ "$status" -eq 3 && "$(cat "$tmp/pointer.err")" == *"SUM-01"* && ! -e "$PACK_REVIEWS_DIR/pointer.md" ]]; then
    pass
else
    fail "a summary with a hidden pointer was not refused: status $status"
fi

# Case 5: a summary that repeats a canary is refused.
canary_summary="$tmp/canary.md"
printf 'The value %s appears here.\n' "$CANARY" > "$canary_summary"
status=0
"$digest" publish-summary canary "$canary_summary" >/dev/null 2>"$tmp/canary.err" || status=$?
if [[ "$status" -eq 3 && ! -e "$PACK_REVIEWS_DIR/canary.md" ]]; then pass; else fail "a summary with a canary was published"; fi

# Case 6: a summary that names a private file is refused.
name_summary="$tmp/name.md"
printf 'The reference is in reference/solution.py.\n' > "$name_summary"
status=0
"$digest" publish-summary name "$name_summary" >/dev/null 2>&1 || status=$?
if [[ "$status" -eq 3 ]]; then pass; else fail "a summary naming reference/solution.py was not refused"; fi

# Case 7: a slug that could escape the folder is refused.
status=0
"$digest" prepare "../demo" "$assembled" "$reports" >/dev/null 2>&1 || status=$?
if [[ "$status" -eq 2 ]]; then pass; else fail "a bad slug was accepted: status $status"; fi

printf 'pack-review-digest tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
