#!/usr/bin/env bats
# Tests for infra/ansible/roles/caddy_site/files/counterspell-caddy-site (deployer agent, shared host).
# The script appends one marked Counterspell block to the shared Caddyfile IN PLACE (same inode), after a
# backup, then validates and reloads the running Caddy. It never recreates or restarts the container and
# never touches existing lines. docker and curl are stubs; nothing contacts a real service.

SCRIPT="$BATS_TEST_DIRNAME/../../../infra/ansible/roles/caddy_site/files/counterspell-caddy-site"

setup() {
  T="$(mktemp -d)"
  export STUB_LOG="$T/calls.log"; : > "$STUB_LOG"
  export CADDY_FILE="$T/Caddyfile"
  export CADDY_CONTAINER="proxy-caddy-1"
  export HEALTH_URL="https://api.example/health"
  export BACKUP_STAMP="20261116-120000"
  export HEALTH_SETTLE_SECS=0
  export PATH="$T/bin:$PATH"
  mkdir -p "$T/bin"
  # docker stub: records calls; `caddy validate` fails when the file holds the word BROKEN or FAIL_VALIDATE is set.
  cat > "$T/bin/docker" <<'EOF'
#!/usr/bin/env bash
echo "docker $*" >> "$STUB_LOG"
if [[ "$*" == *"caddy validate"* ]]; then
  if [[ -n "${FAIL_VALIDATE:-}" ]] || grep -q BROKEN "$CADDY_FILE"; then echo "validate failed" >&2; exit 1; fi
fi
if [[ "$*" == *"caddy reload"* && -n "${FAIL_RELOAD:-}" ]]; then exit 1; fi
exit 0
EOF
  # curl stub: prints the status code from HEALTH_CODE_FILE (changes during a run when the file is rewritten).
  cat > "$T/bin/curl" <<'EOF'
#!/usr/bin/env bash
echo "curl $*" >> "$STUB_LOG"
cat "$HEALTH_CODE_FILE"
EOF
  chmod +x "$T/bin/docker" "$T/bin/curl"
  export HEALTH_CODE_FILE="$T/health"; printf '200' > "$HEALTH_CODE_FILE"
  printf 'api.example {\n    reverse_proxy trading-api:8000\n}\n' > "$CADDY_FILE"
  cp "$CADDY_FILE" "$T/original"
  export CADDY_BLOCK=$'new.example {\n    reverse_proxy counterspell-server:8080\n}'
}

teardown() { rm -rf "$T"; }

inode() { stat -c %i "$1"; }

@test "add appends the marked block in place: same inode, original bytes untouched, backup written" {
  before="$(inode "$CADDY_FILE")"
  run bash "$SCRIPT" add
  [ "$status" -eq 0 ]
  [ "$(inode "$CADDY_FILE")" = "$before" ]
  # The original content is an exact prefix of the new file (Evorayn's lines are not touched).
  cmp -n "$(stat -c %s "$T/original")" "$T/original" "$CADDY_FILE"
  grep -qx '# BEGIN counterspell' "$CADDY_FILE"
  grep -qx '# END counterspell' "$CADDY_FILE"
  grep -q 'reverse_proxy counterspell-server:8080' "$CADDY_FILE"
  cmp "$T/original" "$CADDY_FILE.bak-20261116-120000"
}

@test "add validates first, then reloads, and never restarts or recreates the container" {
  run bash "$SCRIPT" add
  [ "$status" -eq 0 ]
  grep -q "docker exec proxy-caddy-1 caddy validate" "$STUB_LOG"
  grep -q "docker exec proxy-caddy-1 caddy reload" "$STUB_LOG"
  validate_line="$(grep -n 'caddy validate' "$STUB_LOG" | head -1 | cut -d: -f1)"
  reload_line="$(grep -n 'caddy reload' "$STUB_LOG" | head -1 | cut -d: -f1)"
  [ "$validate_line" -lt "$reload_line" ]
  ! grep -Eq "docker (restart|stop|start|rm|kill|compose|run|create)" "$STUB_LOG"
}

@test "a second add with the same block changes nothing and writes no second backup" {
  bash "$SCRIPT" add
  cp "$CADDY_FILE" "$T/after-first"
  : > "$STUB_LOG"
  run bash "$SCRIPT" add
  [ "$status" -eq 0 ]
  cmp "$T/after-first" "$CADDY_FILE"
  [ "$(grep -c '^# BEGIN counterspell$' "$CADDY_FILE")" -eq 1 ]
  ! grep -q "caddy reload" "$STUB_LOG"
}

@test "a changed block replaces only the marked block, in place" {
  bash "$SCRIPT" add
  before="$(inode "$CADDY_FILE")"
  export CADDY_BLOCK=$'new.example {\n    reverse_proxy counterspell-server:9090\n}'
  export BACKUP_STAMP="20261116-130000"
  run bash "$SCRIPT" add
  [ "$status" -eq 0 ]
  [ "$(inode "$CADDY_FILE")" = "$before" ]
  grep -q 'counterspell-server:9090' "$CADDY_FILE"
  ! grep -q 'counterspell-server:8080' "$CADDY_FILE"
  [ "$(grep -c '^# BEGIN counterspell$' "$CADDY_FILE")" -eq 1 ]
  cmp -n "$(stat -c %s "$T/original")" "$T/original" "$CADDY_FILE"
}

@test "a failed validate restores the backup byte for byte, keeps the inode, and does not reload" {
  before="$(inode "$CADDY_FILE")"
  export FAIL_VALIDATE=1
  run bash "$SCRIPT" add
  [ "$status" -ne 0 ]
  cmp "$T/original" "$CADDY_FILE"
  [ "$(inode "$CADDY_FILE")" = "$before" ]
  ! grep -q "caddy reload" "$STUB_LOG"
}

@test "a failed reload restores the backup and asks Caddy to reload the restored file" {
  export FAIL_RELOAD=1
  run bash "$SCRIPT" add
  [ "$status" -ne 0 ]
  cmp "$T/original" "$CADDY_FILE"
  [ "$(grep -c 'caddy reload' "$STUB_LOG")" -eq 2 ]
}

@test "a changed health status after the reload restores the backup (Evorayn must answer the same)" {
  # The stub reports 200 before and 502 once the block is in the file.
  cat > "$T/bin/curl" <<'EOF'
#!/usr/bin/env bash
if grep -q '# BEGIN counterspell' "$CADDY_FILE"; then printf '502'; else printf '200'; fi
EOF
  run bash "$SCRIPT" add
  [ "$status" -ne 0 ]
  cmp "$T/original" "$CADDY_FILE"
  [[ "$output" == *"health"* ]]
}

@test "remove deletes exactly the marked block, in place, and leaves Evorayn's lines" {
  bash "$SCRIPT" add
  before="$(inode "$CADDY_FILE")"
  export BACKUP_STAMP="20261116-140000"
  run bash "$SCRIPT" remove
  [ "$status" -eq 0 ]
  [ "$(inode "$CADDY_FILE")" = "$before" ]
  ! grep -q 'counterspell' "$CADDY_FILE"
  cmp "$T/original" "$CADDY_FILE"
}

@test "plan changes nothing and reports whether an add is needed" {
  run bash "$SCRIPT" plan
  [ "$status" -eq 0 ]
  [[ "$output" == *"would append"* ]]
  cmp "$T/original" "$CADDY_FILE"
  ! ls "$CADDY_FILE".bak-* 2>/dev/null
  bash "$SCRIPT" add
  run bash "$SCRIPT" plan
  [[ "$output" == *"unchanged"* ]]
}

@test "a missing Caddyfile or an empty block is refused before anything is written" {
  rm "$CADDY_FILE"
  run bash "$SCRIPT" add
  [ "$status" -ne 0 ]
  printf 'x {\n}\n' > "$CADDY_FILE"
  export CADDY_BLOCK=""
  run bash "$SCRIPT" add
  [ "$status" -ne 0 ]
  ! ls "$CADDY_FILE".bak-* 2>/dev/null
}
