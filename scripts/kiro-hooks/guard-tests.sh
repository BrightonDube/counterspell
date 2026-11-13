#!/usr/bin/env bash
# PreToolUse guard for test integrity (see .kiro/steering/tdd-and-quality.md).
#
# Blocks (exit 2) only two things, and only in source files (.rs .py .js .ts ...):
#   1. one edit that removes more test markers than it adds back (a net loss of tests);
#   2. one edit that adds a skip/ignore marker, except Rust #[ignore] on pg_* / valkey_*
#      tests, which is the repo's database-test convention.
# Deleting a source file that contains tests is treated as removing those tests.
#
# Escape hatch for a real requirement change: put `test-change: <REQ-ID>` in the edit
# (a comment is fine), where <REQ-ID> exists in docs/requirements.md, for example
# `// test-change: VAL-3`. Bypasses are logged to docs/kiro-log/test-guard.log.
#
# This guard fails open: a missing jq, an odd event, or an unreadable file allows the edit.
# Not covered: edits made through the shell (sed -i, rm, git checkout).
set -uo pipefail

event=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

tool=$(jq -r '.tool_name // ""' <<<"$event" 2>/dev/null || true)
get() { jq -r "(.tool_input // {}) | ($1) // \"\"" <<<"$event" 2>/dev/null || true; }

path=$(get '.path // .filePath // .file_path // .targetFile')
[[ -n "$path" ]] || exit 0

case "$path" in
  *.rs) lang=rs ;;
  *.py) lang=py ;;
  *.js | *.mjs | *.cjs | *.jsx | *.ts | *.tsx) lang=js ;;
  *) exit 0 ;;
esac

# Test markers (counted as matches) and skip markers, per language.
case "$lang" in
  rs)
    test_re='#\[([a-z_]+::)?test(\]|\()|#\[(rstest|test_case)|proptest![[:space:]]*\{'
    skip_re='#\[ignore'
    ;;
  py)
    test_re='^[[:space:]]*(async[[:space:]]+)?def[[:space:]]+test[A-Za-z0-9_]*[[:space:]]*\('
    skip_re='@(pytest\.mark\.(skip|skipif|xfail)|unittest\.(skip|skipIf|skipUnless|expectedFailure))|pytest\.skip\(|\.skipTest\('
    ;;
  js)
    test_re='(^|[^[:alnum:]_.$])(it|test)(\.(each|only|concurrent))?[[:space:]]*[(`]'
    skip_re='(^|[^[:alnum:]_.$])(xit|xtest|xdescribe)[[:space:]]*\(|(it|test|describe)\.(skip|todo|fails)[[:space:]]*[(`]'
    ;;
esac

count() { printf '%s\n' "$1" | grep -Eo -- "$2" | wc -l | tr -d '[:space:]'; }

# Rust: count #[ignore] attributes whose next `fn` is not named pg_* or valkey_*.
bad_rust_ignores() {
  printf '%s\n' "$1" | awk '
    /#\[ignore/ { pending++ }
    pending > 0 && match($0, /fn[ \t]+[A-Za-z0-9_]+/) {
      name = substr($0, RSTART, RLENGTH); sub(/^fn[ \t]+/, "", name)
      if (name !~ /^(pg|valkey)_/) bad += pending
      pending = 0
    }
    END { bad += pending; print bad + 0 }'
}
skip_count() { if [[ $lang == rs ]]; then bad_rust_ignores "$1"; else count "$1" "$skip_re"; fi; }

# Work out what the edit does: delete a file, add text, replace a snippet, or overwrite.
sub=$(get '.command')
old_text=$(get '.oldStr // .old_str')
new_text=$(get '.newStr // .new_str // .text // .file_text // .content')
explanation=$(get '.explanation')

resolved=''
if [[ -f "$path" ]]; then
  resolved=$path
elif command -v wslpath >/dev/null 2>&1; then
  converted=$(wslpath -u "$path" 2>/dev/null || true)
  if [[ -n "$converted" && -f "$converted" ]]; then resolved=$converted; fi
fi

kind=write
case "$tool" in
  *delete*) kind=delete ;;
  *append*) kind=add ;;
esac
case "$sub" in
  insert | append) kind=add ;;
esac
if [[ $kind == write && -n "$old_text" ]]; then kind=replace; fi

case "$kind" in
  add) old_text='' ;;
  delete)
    [[ -n "$resolved" ]] || exit 0
    old_text=$(cat "$resolved" 2>/dev/null || true)
    new_text=''
    ;;
  write)
    old_text=''
    if [[ -n "$resolved" ]]; then old_text=$(cat "$resolved" 2>/dev/null || true); fi
    ;;
esac

old_tests=$(count "$old_text" "$test_re")
new_tests=$(count "$new_text" "$test_re")
old_skips=$(skip_count "$old_text")
new_skips=$(skip_count "$new_text")
for n in "$old_tests" "$new_tests" "$old_skips" "$new_skips"; do
  [[ $n =~ ^[0-9]+$ ]] || exit 0
done

removed=$((old_tests - new_tests))
added_skips=$((new_skips - old_skips))
if [[ $removed -le 0 && $added_skips -le 0 ]]; then exit 0; fi

# Escape hatch: `test-change: <REQ-ID>` in the new text (or the delete explanation).
ids=$(printf '%s\n%s\n' "$new_text" "$explanation" \
  | grep -Eo 'test-change:[[:space:]]*[A-Z]{2,5}-[0-9]+' | sed 's/^.*:[[:space:]]*//' || true)
for id in $ids; do
  if grep -Eq "(^|[^A-Z0-9-])${id}([^0-9]|$)" docs/requirements.md 2>/dev/null; then
    printf '%s %s %s test-change: %s\n' "$(date -u +%FT%TZ)" "$tool" "$path" "$id" \
      >> docs/kiro-log/test-guard.log 2>/dev/null || true
    exit 0
  fi
done

if [[ $removed -gt 0 ]]; then
  {
    printf 'blocked: this %s would remove %s test(s) from %s (before: %s, after: %s).\n' \
      "$kind" "$removed" "$path" "$old_tests" "$new_tests"
    printf '%s\n' 'Do not delete tests to get green. Fix the code so the test passes.'
    printf '%s\n' 'Moving or renaming tests in the same edit is fine. If the requirement itself changed,'
    printf '%s\n' 'add a comment "test-change: <REQ-ID>" (an ID from docs/requirements.md) in the edit'
    printf '%s\n' 'and say why in your summary. If you cannot fix it cleanly, leave the test failing and'
    printf '%s\n' 'record it in docs/kiro-log/<spec>-blockers.md.'
  } >&2
  exit 2
fi

{
  printf 'blocked: this %s adds a skip/ignore marker to a test in %s.\n' "$kind" "$path"
  printf '%s\n' 'Do not disable a test to get green. Fix the code so the test passes.'
  printf '%s\n' 'Rust #[ignore] is allowed only on tests named pg_* or valkey_* (they need a database);'
  printf '%s\n' 'put the "fn pg_..." line in the same edit. If the requirement changed, add a comment'
  printf '%s\n' '"test-change: <REQ-ID>" (an ID from docs/requirements.md). Otherwise leave the test'
  printf '%s\n' 'failing and record it in docs/kiro-log/<spec>-blockers.md.'
} >&2
exit 2
