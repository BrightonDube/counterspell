#!/usr/bin/env bash
# pack-batch-test.sh - exercises pack-batch.sh on fixture slugs in a temporary
# directory and asserts the pipeline's safety rails hold.
set -euo pipefail
REPO=$(cd "$(dirname "$0")/../.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/packs/ok-pack" "$TMP/private/packs/ok-pack/reference" "$TMP/bin" "$TMP/.kiro/pipeline/state" "$TMP/docs/kiro-log"
cat > "$TMP/packs/ok-pack/meta.toml" <<'TOML'
slug = "ok-pack"
title = "Ok Pack"
track = "arrays"
concepts = ["arrays"]
difficulty = "easy"
pack_version = 1
status = "draft"
TOML

printf "fixture\n" > "$TMP/private/packs/ok-pack/reference/solution.py"
fail=0

# A stub grimoire that always passes: the batch must reach awaiting_approval.
cat > "$TMP/bin/counterspell-grimoire" <<'SH'
#!/usr/bin/env bash
[[ "$1" == validate && "$3" == --deny-warnings && -f "$2/reference/solution.py" ]] || exit 1
echo "report codes: none"
exit 0
SH
chmod +x "$TMP/bin/counterspell-grimoire"

export PATH="$TMP/bin:$PATH"
export PACK_BATCH_ROOT="$TMP"
export COUNTERSPELL_PACKS_PRIVATE_ROOT="$TMP/private"
bash "$REPO/scripts/pack-batch.sh" ok-pack > /dev/null

state="$TMP/.kiro/pipeline/state/ok-pack.toml"
if grep -q 'awaiting_approval' "$state"; then
  echo "PASS: state reached awaiting_approval"
else
  echo "FAIL: state did not reach awaiting_approval"
  cat "$state"
  fail=1
fi

if grep -q 'published' "$state"; then
  echo "FAIL: 'published' appeared in the state file"
  fail=1
fi

if find "$TMP" -name 'approval.toml' | grep -q .; then
  echo "FAIL: approval.toml was written"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "pack-batch-test: all assertions passed (1 directory, 4 checks)"
fi
exit "$fail"
