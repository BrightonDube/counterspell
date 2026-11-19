#!/usr/bin/env bash
# Checks the shared-host playbook (deploy-ops, deployer `prepare`).
#  - it runs only app, sandbox and caddy_site, never base or docker;
#  - it refuses to run unless shared_host is true, before any role task;
#  - the Counterspell Caddy site file proxies to the server container and sets no `Server` header.
# Nothing here touches a host: the refusal test fails on its first task, and the render writes to a temp dir.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANSIBLE_DIR="$HERE/.."
PLAYBOOK="$ANSIBLE_DIR/counterspell-shared.yml"
RENDER_DIR="$(mktemp -d)"
trap 'rm -rf "$RENDER_DIR"' EXIT
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

if [ ! -f "$PLAYBOOK" ]; then
  fail "counterspell-shared.yml does not exist"
  exit 1
fi

# Roles named by the playbook (the `role:` lines).
roles="$(grep -E '^\s+- role:' "$PLAYBOOK" | sed -E 's/.*role:\s*//' | sort | tr '\n' ' ')"
if [ "$roles" = "app caddy_site sandbox " ]; then
  pass "playbook runs exactly app, caddy_site and sandbox"
else
  fail "playbook roles are: $roles"
fi
if grep -Eq '^\s+- role:\s*(base|docker)\s*$' "$PLAYBOOK"; then
  fail "playbook pulls in base or docker"
else
  pass "playbook never pulls in base or docker"
fi

# The guard must refuse when shared_host is not true, and must do it before any role runs.
inventory="$RENDER_DIR/inventory.yml"
cat > "$inventory" <<'EOF'
all:
  children:
    counterspell:
      hosts:
        guard-test:
          ansible_connection: local
          shared_host: false
          caddy_site_address: guard.example
EOF
output="$(ansible-playbook -i "$inventory" "$PLAYBOOK" 2>&1 || true)"
if grep -q 'shared_host' <<<"$output" && grep -q 'FAILED\|failed=1' <<<"$output" && ! grep -q 'TASK \[app :' <<<"$output"; then
  pass "playbook refuses when shared_host is false, before the app role"
else
  fail "playbook did not refuse cleanly: $output"
fi

# Render the Caddy site file and check it.
ansible-playbook "$HERE/render_caddy_site_test.yml" -e "render_dir=$RENDER_DIR" > /dev/null
site="$RENDER_DIR/counterspell.caddy"
if grep -Fq '15-204-92-66.sslip.io {' "$site"; then pass "site block uses the site address"; else fail "site address missing"; fi
if grep -Fq 'reverse_proxy counterspell-server:8080' "$site"; then pass "site proxies to the server container"; else fail "reverse_proxy target wrong"; fi
if grep -Fq 'respond /api/ready 404' "$site"; then pass "readiness is not public"; else fail "readiness route not blocked"; fi
if grep -Fq 'flush_interval -1' "$site"; then pass "SSE route is unbuffered"; else fail "SSE flush missing"; fi
if grep -Eq '^\s*-Server' "$site"; then pass "Server header removed"; else fail "Server header not removed"; fi
if grep -Eq 'import|admin|auto_https|acme' "$site"; then fail "site file contains global or import directives"; else pass "site file holds a single site block only"; fi

# The sandbox role must use the scoped rootlesskit AppArmor profile, never the host-wide sysctl (R3.8, Q12).
sandbox_tasks="$ANSIBLE_DIR/roles/sandbox/tasks/main.yml"
sandbox_defaults="$ANSIBLE_DIR/roles/sandbox/defaults/main.yml"
if grep -qF 'apparmor_parser -r /etc/apparmor.d/rootlesskit' "$sandbox_tasks"; then
  pass "sandbox role loads the scoped rootlesskit AppArmor profile"
else
  fail "sandbox role does not load /etc/apparmor.d/rootlesskit"
fi
if grep -qF '/etc/apparmor.d/rootlesskit' "$sandbox_tasks" && grep -q 'apparmor' "$sandbox_defaults"; then
  pass "sandbox role installs apparmor and checks the profile file"
else
  fail "sandbox role does not ensure the apparmor package and profile file"
fi
if grep -q '/proc/sys/kernel/apparmor_restrict_unprivileged_userns' "$sandbox_tasks"; then
  fail "sandbox role still reads the host-wide AppArmor userns sysctl"
else
  pass "sandbox role leaves kernel.apparmor_restrict_unprivileged_userns alone"
fi

if [ "$failures" -gt 0 ]; then
  echo "$failures check(s) failed" >&2
  exit 1
fi
