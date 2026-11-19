#!/usr/bin/env bash
# Checks the app role's service env templates (deploy-ops, deployer `prepare`).
#  - each of postgres, migrate, server and judge renders a complete env file;
#  - the files carry the keys the compose services and binaries read;
#  - the role writes them mode 0600 and never logs their content.
# Nothing here touches a host or a real secret: render_app_env_test.yml uses fixture values.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANSIBLE_DIR="$HERE/.."
RENDER_DIR="$(mktemp -d)"
trap 'rm -rf "$RENDER_DIR"' EXIT
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

ansible-playbook "$HERE/render_app_env_test.yml" -e "render_dir=$RENDER_DIR" > /dev/null

check_grep() {
  local file="$1" pattern="$2" label="$3"
  if grep -qF "$pattern" "$file"; then pass "$label"; else fail "$label ($file lacks '$pattern')"; fi
}
check_absent() {
  local file="$1" pattern="$2" label="$3"
  if grep -qF "$pattern" "$file"; then fail "$label ($file has '$pattern')"; else pass "$label"; fi
}

# postgres.env feeds the postgres container's first initdb.
pg="$RENDER_DIR/postgres.env"
check_grep "$pg" "POSTGRES_USER=counterspell" "postgres.env sets the superuser name"
check_grep "$pg" "POSTGRES_DB=counterspell" "postgres.env sets the database"
check_grep "$pg" "POSTGRES_PASSWORD=fixture-postgres-pw" "postgres.env carries the vault password"
check_grep "$pg" "CS_SERVER_PASSWORD=fixture-server-pw" "postgres.env carries the cs_server password for initdb"
check_grep "$pg" "CS_JUDGE_PASSWORD=fixture-judge-pw" "postgres.env carries the cs_judge password for initdb"

# migrate.env is the sqlx CLI's DATABASE_URL; it runs as the database owner.
mg="$RENDER_DIR/migrate.env"
check_grep "$mg" "DATABASE_URL=postgres://counterspell:fixture-postgres-pw@postgres:5432/counterspell" "migrate.env points at the owner role"

# server.env feeds counterspell-server (ServerConfig plus AuthConfig).
sv="$RENDER_DIR/server.env"
check_grep "$sv" "APP_ENV=production" "server.env pins production"
check_grep "$sv" "PUBLIC_BASE_URL=https://counterspell.example" "server.env builds URLs from the site address"
check_grep "$sv" "DATABASE_URL=postgres://cs_server:fixture-server-pw@postgres:5432/counterspell" "server.env uses the cs_server login"
check_grep "$sv" "VALKEY_URL=redis://valkey:6379" "server.env points at the internal Valkey"
check_grep "$sv" "SESSION_SECRET=fixture-session-secret-0123456789" "server.env carries the vault session secret"
check_grep "$sv" "GITHUB_OAUTH_CLIENT_ID=fixture-gh-id" "server.env carries the GitHub client id"
check_grep "$sv" "GITHUB_OAUTH_CLIENT_SECRET=fixture-gh-secret" "server.env carries the GitHub secret"
check_grep "$sv" "GOOGLE_OAUTH_CLIENT_ID=fixture-google-id" "server.env carries the Google client id"
check_grep "$sv" "GOOGLE_OAUTH_CLIENT_SECRET=fixture-google-secret" "server.env carries the Google secret"

# judge.env feeds counterspell-judge (its own config plus DATABASE_URL).
jg="$RENDER_DIR/judge.env"
check_grep "$jg" "DATABASE_URL=postgres://cs_judge:fixture-judge-pw@postgres:5432/counterspell" "judge.env uses the cs_judge login"
check_grep "$jg" "JUDGE_CONCURRENCY=2" "judge.env sets the concurrency cap"
check_grep "$jg" "LEASE_SECS=30" "judge.env sets the lease"
check_grep "$jg" "RENEW_SECS=10" "judge.env sets the renew interval"

# No template marker may survive rendering, and no file may carry an empty secret slot.
for f in "$pg" "$mg" "$sv" "$jg"; do
  check_absent "$f" "{{" "no unrendered marker in $(basename "$f")"
  check_absent "$f" "=}}" "no unrendered tail in $(basename "$f")"
done

# The role must write the files owner-only and must not log their content.
tasks="$ANSIBLE_DIR/roles/app/tasks/main.yml"
if grep -q 'mode: "0600"' "$tasks" && grep -q 'no_log: true' "$tasks"; then
  pass "app role writes env files mode 0600 with no_log"
else
  fail "app role does not write env files mode 0600 with no_log"
fi

if [ "$failures" -gt 0 ]; then
  echo "$failures check(s) failed" >&2
  exit 1
fi
echo "all app env checks passed"
