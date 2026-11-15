#!/usr/bin/env bats
# compose.sha256 written by the app role (infra/ansible/roles/app) must be accepted by the same
# `sha256sum --check` that counterspell-deploy runs before any pull (deploy-ops T12, R7.13).

REPO="$BATS_TEST_DIRNAME/../../.."

setup() {
  TEST_ROOT="$(mktemp -d)"
  APP="$TEST_ROOT/app"
  mkdir -p "$APP"
  printf 'services: {}\n' > "$APP/docker-compose.yml"
  printf 'services: {}\n' > "$APP/compose.prod.yml"
  printf 'counterspell.example {}\n' > "$APP/Caddyfile"

  # Build the stat results the way the app role does: one entry per copied file, and extra vars
  # for the render (a JSON file, so the nested structure is passed intact).
  python3 - "$APP" "$TEST_ROOT/vars.json" <<'PY'
import hashlib, json, os, sys
app, out = sys.argv[1], sys.argv[2]
results = []
for name in ["docker-compose.yml", "compose.prod.yml", "Caddyfile"]:
    with open(os.path.join(app, name), "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    results.append({"item": {"dest": name}, "stat": {"checksum": digest}})
with open(out, "w") as handle:
    json.dump({"render_dir": app, "app_compose_stat": {"results": results}}, handle)
PY

  ansible-playbook "$REPO/infra/ansible/tests/render_app_test.yml" -e "@$TEST_ROOT/vars.json" > /dev/null
}

teardown() {
  rm -rf "$TEST_ROOT"
}

@test "the rendered record is accepted by the deploy script's sha256sum check" {
  run bash -c "cd '$APP' && sha256sum --check --quiet compose.sha256"
  [ "$status" -eq 0 ]
}

@test "a changed compose file is refused by the same check (R7.13)" {
  printf 'services: { changed: {} }\n' > "$APP/compose.prod.yml"
  run bash -c "cd '$APP' && sha256sum --check --quiet compose.sha256"
  [ "$status" -ne 0 ]
}

@test "every record line is '<64 hex>  <file name>' for the three copied files" {
  run grep -cE '^[0-9a-f]{64}  (docker-compose\.yml|compose\.prod\.yml|Caddyfile)$' "$APP/compose.sha256"
  [ "$status" -eq 0 ]
  [ "$output" -eq 3 ]
}
