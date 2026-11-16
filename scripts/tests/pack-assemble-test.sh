#!/usr/bin/env bash
# Behaviour tests for scripts/pack-assemble.sh (PA07, PIP-31, PIP-32, PIP-34, D3).
#
# Every case uses temporary directories and synthetic data. Nothing touches the real
# private repository, and no case pushes or calls the network.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
assemble="$script_dir/../pack-assemble.sh"

# A fixed canary that stands in for hidden data. It must never reach any output.
readonly CANARY="CANARY-HIDDEN-pa7"

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

passed=0
failed=0

pass() { passed=$((passed + 1)); }
fail() { failed=$((failed + 1)); printf 'FAIL: %s\n' "$*" >&2; }

# Creates a public and a private tree for slug `demo` under $tmp/<name>.
make_trees() {
    local name="$1"
    local public="$tmp/$name/public" private="$tmp/$name/private"
    mkdir -p "$public/packs/demo" "$private/packs/demo/reference" "$private/packs/demo/incorrect"
    printf '# Demo\n' > "$public/packs/demo/problem.md"
    printf 'slug = "demo"\nstatus = "draft"\n' > "$public/packs/demo/meta.toml"
    printf 'slug = "demo"\n' > "$public/packs/demo/hints.toml"
    printf 'def solve(values):\n    return 0\n' > "$private/packs/demo/reference/solution.py"
    printf '[[case]]\nexpected = "%s"\n' "$CANARY" > "$private/packs/demo/hidden.toml"
    printf 'print("wrong")\n' > "$private/packs/demo/incorrect/off_by_one.py"
}

# Runs the assembler with its output under $tmp/<name>/assembly.
# Sets $status (the exit code), $out (stdout) and $err (stderr).
run_assemble() {
    local name="$1" slug="$2" public="$3" private="$4"
    status=0
    out="$(PACK_ASSEMBLY_ROOT="$tmp/$name/assembly" "$assemble" "$slug" "$public" "$private" 2>"$tmp/$name.err")" || status=$?
    err="$(cat "$tmp/$name.err")"
}

# Case 1: a normal assembly passes, copies both trees and prints a 64-character hash.
make_trees normal
run_assemble normal demo "$tmp/normal/public" "$tmp/normal/private"
if [[ "$status" -eq 0 ]] && [[ "$out" =~ ^[0-9a-f]{64}$ ]]; then pass; else fail "normal assembly: status $status, out '$out'"; fi
for file in problem.md meta.toml hints.toml hidden.toml reference/solution.py incorrect/off_by_one.py; do
    if [[ -f "$tmp/normal/assembly/demo/$file" ]]; then pass; else fail "assembled tree lacks $file"; fi
done

# Case 2: no hidden value appears in stdout, stderr or the manifest.
manifest="$tmp/normal/assembly/demo.manifest"
if [[ "$out$err" != *"$CANARY"* ]] && { [[ ! -f "$manifest" ]] || ! grep -q "$CANARY" "$manifest"; }; then
    pass
else
    fail "a canary leaked into output or the manifest"
fi

# Case 3: the manifest lists the six files, sorted by path.
if [[ -f "$manifest" ]] && awk '{print $2}' "$manifest" | LC_ALL=C sort -c 2>/dev/null \
    && [[ "$(wc -l < "$manifest")" -eq 6 ]]; then
    pass
else
    fail "manifest is not sorted or does not list six files"
fi

# Case 4: the same inputs give the same hash, and one changed byte gives a different hash.
make_trees determinism
run_assemble determinism demo "$tmp/determinism/public" "$tmp/determinism/private"
first="$out"
run_assemble determinism demo "$tmp/determinism/public" "$tmp/determinism/private"
if [[ "$first" == "$out" ]]; then pass; else fail "the same inputs gave different hashes"; fi
printf 'def solve(values):\n    return 1\n' > "$tmp/determinism/private/packs/demo/reference/solution.py"
run_assemble determinism demo "$tmp/determinism/public" "$tmp/determinism/private"
if [[ "$status" -eq 0 && "$out" != "$first" ]]; then
    pass
else
    fail "a changed private file did not change the content hash"
fi

# Case 5: a private file placed in the public tree is refused with LAY-05, and nothing is assembled.
make_trees leak
cp "$tmp/leak/private/packs/demo/hidden.toml" "$tmp/leak/public/packs/demo/hidden.toml"
run_assemble leak demo "$tmp/leak/public" "$tmp/leak/private"
if [[ "$status" -eq 3 && "$err" == *"LAY-05"* && ! -e "$tmp/leak/assembly/demo" ]]; then
    pass
else
    fail "private file in public tree: status $status, err '$err'"
fi

# Case 6: a missing private tree stops the pack with PIP-34.
make_trees missing
rm -rf -- "$tmp/missing/private/packs/demo"
run_assemble missing demo "$tmp/missing/public" "$tmp/missing/private"
if [[ "$status" -eq 4 && "$err" == *"PIP-34"* ]]; then pass; else fail "missing private tree: status $status, err '$err'"; fi

# Case 7: a slug that could escape the directory is refused before any path is built.
make_trees traversal
run_assemble traversal "../demo" "$tmp/traversal/public" "$tmp/traversal/private"
if [[ "$status" -eq 2 ]]; then pass; else fail "path traversal slug: status $status"; fi

# Case 8: a symbolic link in the public tree is refused.
make_trees symlink
ln -s "$tmp/symlink/private/packs/demo/hidden.toml" "$tmp/symlink/public/packs/demo/link.toml"
run_assemble symlink demo "$tmp/symlink/public" "$tmp/symlink/private"
if [[ "$status" -eq 3 && "$err" == *"symbolic link"* ]]; then pass; else fail "symlink: status $status, err '$err'"; fi

# Case 9: a file present in both trees is refused rather than overwritten.
make_trees collision
printf 'notes\n' > "$tmp/collision/private/packs/demo/problem.md"
run_assemble collision demo "$tmp/collision/public" "$tmp/collision/private"
if [[ "$status" -eq 3 && "$err" == *"LAY-05"* ]]; then pass; else fail "collision: status $status, err '$err'"; fi

# Case 10: running without the three arguments is a usage error.
status=0
"$assemble" >/dev/null 2>&1 || status=$?
if [[ "$status" -eq 2 ]]; then pass; else fail "usage: status $status"; fi

printf 'pack-assemble tests: %d passed, %d failed\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
