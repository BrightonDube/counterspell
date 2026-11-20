#!/usr/bin/env bash
# Shape test for the production Compose stack on the shared host (deploy-ops T14/T15, R4.1 to R4.10,
# R12.4, P1, P2, P8). The shared host already runs a reverse proxy, so Counterspell runs no Caddy of its
# own: compose.shared.yml removes it and joins the server to the existing counterspell_edge network, which is external.
#
# It renders the stack with `docker compose config` (no containers start, nothing is pulled), then asserts
# on the JSON with jq. COUNTERSPELL_SECRETS_DIR (host.env) is pointed at placeholder fixtures in a
# temporary copy, so no host file is read.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
FIXTURES="$ROOT/deploy/vps/tests/fixtures"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

# A temporary copy of the overlay, with the env file paths moved to the fixtures.
mkdir -p "$TMP/env"
for service in postgres migrate server judge; do
  cp "$FIXTURES/$service.env.example" "$TMP/env/$service.env"
done
cp "$FIXTURES/host.env.example" "$TMP/env/host.env"
sed -i "s#^COUNTERSPELL_SECRETS_DIR=.*#COUNTERSPELL_SECRETS_DIR=$TMP/env#" "$TMP/env/host.env"
cp "$ROOT/deploy/vps/compose.prod.yml" "$TMP/compose.prod.yml"
cp "$ROOT/deploy/vps/compose.shared.yml" "$TMP/compose.shared.yml"

cat > "$TMP/deploy.env" <<'EOF'
COUNTERSPELL_SHA=0000000000000000000000000000000000000000
COUNTERSPELL_SERVER_IMAGE=ghcr.io/brightondube/counterspell-server@sha256:1111111111111111111111111111111111111111111111111111111111111111
COUNTERSPELL_JUDGE_IMAGE=ghcr.io/brightondube/counterspell-judge@sha256:2222222222222222222222222222222222222222222222222222222222222222
COUNTERSPELL_MIGRATE_IMAGE=ghcr.io/brightondube/counterspell-migrate@sha256:3333333333333333333333333333333333333333333333333333333333333333
EOF

docker compose --project-directory "$ROOT" \
  -f "$ROOT/docker-compose.yml" -f "$TMP/compose.prod.yml" -f "$TMP/compose.shared.yml" \
  --env-file "$TMP/deploy.env" --env-file "$TMP/env/host.env" \
  config --no-env-resolution --format json > "$TMP/config.json"

# Asserts that a jq expression is true for the rendered stack.
check() {
  local description="$1" expression="$2"
  if jq -e "$expression" "$TMP/config.json" > /dev/null; then
    pass "$description"
  else
    fail "$description"
  fi
}

# P1: only judge mounts a sandbox socket, and no service mounts the main daemon socket.
check "P1 only judge mounts a Docker socket" \
  '[.services | to_entries[] | select(any((.value.volumes // [])[]; (.source // "") | test("docker.sock"))) | .key] == ["judge"]'
check "P1 no service mounts /var/run/docker.sock" \
  '[.services[] | (.volumes // [])[] | .source // ""] | any(. == "/var/run/docker.sock") | not'

# P2 (shared host): Counterspell runs no Caddy and publishes no port. The shared proxy is the only edge.
check "P2 there is no caddy service" \
  '.services | has("caddy") | not'
check "P2 no service publishes a port" \
  '[.services[] | (.ports // []) | length] | all(. == 0)'
check "P2 only server joins the shared edge network" \
  '[.services | to_entries[] | select((.value.networks // {}) | has("edge")) | .key] == ["server"]'
check "P2 the edge network is the existing external counterspell_edge" \
  '.networks.edge.external == true and .networks.edge.name == "counterspell_edge"'
check "P2 judge, postgres, valkey and migrate are on data only" \
  '[.services.judge, .services.postgres, .services.valkey, .services.migrate] | all((.networks // {}) | keys == ["data"])'
check "P2 the data network is internal" \
  '.networks.data.internal == true'

# R4.9: the server trusts forwarded addresses only from the proxy subnet in host.env.
check "R4.9 server takes TRUSTED_PROXIES from host.env" \
  '.services.server.environment.TRUSTED_PROXIES != null'
check "Server can start when optional AI hint keys are absent" \
  '.services.server.environment.AI_STRICT_CONFIG == "false"'

# P8: judge reads only judge.env, with only the allow-listed keys.
check "P8 judge reads judge.env" \
  '(.services.judge.env_file // []) | map(.path) | any(test("judge.env$"))'
check "P8 judge environment holds only allow-listed keys" \
  '((.services.judge.environment // {}) | keys) - ["DATABASE_URL","JUDGE_CONCURRENCY","LEASE_SECS","RENEW_SECS","LOG_FORMAT","SANDBOX_DOCKER_SOCKET"] == []'

# P1 detail: the judge gets the socket at a fixed path and is told about it.
check "P1 judge mounts the sandbox socket read-write at /run/sandbox/docker.sock and sets SANDBOX_DOCKER_SOCKET" \
  '.services.judge | ((.volumes // []) | any(.target == "/run/sandbox/docker.sock")) and (.environment.SANDBOX_DOCKER_SOCKET == "/run/sandbox/docker.sock")'

# R4.5: agreed memory limits (bytes) for postgres and valkey.
check "R4.5 postgres 1024m and valkey 160m" \
  '(.services.postgres.mem_limit | tonumber) == 1073741824 and (.services.valkey.mem_limit | tonumber) == 167772160'
check "Valkey starts as its service user without capabilities" \
  '.services.valkey | .user == "valkey:valkey" and (.cap_drop | index("ALL") != null)'

# R5.1: each service reads only its own env file.
check "R5.1 postgres, server and migrate read their own env files" \
  '([.services.postgres.env_file[].path] | any(test("postgres.env$"))) and ([.services.server.env_file[].path] | any(test("server.env$"))) and ([.services.migrate.env_file[].path] | any(test("migrate.env$")))'

# R4.6: Postgres never uses trust, anywhere in the rendered stack.
check "R4.6 POSTGRES_HOST_AUTH_METHOD is absent" \
  '[.. | objects | keys[]] | index("POSTGRES_HOST_AUTH_METHOD") == null and ([.. | strings] | any(. == "trust")) == false'

# R4.4: hardening on server, judge and migrate.
for service in server judge migrate; do
  check "R4.4 $service is read-only with dropped capabilities and no-new-privileges" \
    ".services.$service | .read_only == true and (.cap_drop | index(\"ALL\") != null) and ((.security_opt // []) | any(test(\"no-new-privileges\")))"
  check "R4.4 $service has a PID limit and a memory limit" \
    ".services.$service | (.pids_limit != null) and (.mem_limit != null)"
done

# R4.1: every image is pinned by digest.
check "R4.1 every image is pinned by digest" \
  '[.services[] | select(.image != null) | .image | test("@sha256:")] | all'

# R4.7: server starts only after migrate completes.
check "R4.7 server waits for migrate to complete successfully" \
  '.services.server.depends_on.migrate.condition == "service_completed_successfully"'

# R4.8: json-file logs are capped in every service.
check "R4.8 every service caps its json-file log at 10 MB and 5 files" \
  '[.services[] | (.logging.options // {}) | (.["max-size"] == "10m" and .["max-file"] == "5")] | all'

# R4.10: judge gets no value interpolated from the base file's host variables.
check "R4.10 judge has no DATABASE_URL from the base file's interpolation" \
  '(.services.judge.environment // {}) | has("DATABASE_URL") | not'

if ((failures > 0)); then
  echo "$failures check(s) failed" >&2
  exit 1
fi
echo "all Compose shape checks passed"
