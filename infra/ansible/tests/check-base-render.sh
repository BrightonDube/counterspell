#!/usr/bin/env bash
# Renders the base role's templates into a throwaway directory and checks them (deploy-ops T11).
# The sudoers file is checked by visudo; the forced-command line and the sshd drop-in by grep.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RENDER_DIR="$(mktemp -d)"
trap 'rm -rf "$RENDER_DIR"' EXIT
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

ansible-playbook "$HERE/render_base_test.yml" -e "render_dir=$RENDER_DIR" > /dev/null

# R7.12: the sudoers file must parse, and name only the exact deploy script path.
if visudo -cf "$RENDER_DIR/sudoers" > /dev/null; then
  pass "sudoers parses with visudo"
else
  fail "sudoers does not parse"
fi
if grep -Fxq 'deploy ALL=(root) NOPASSWD: /usr/local/sbin/counterspell-deploy' "$RENDER_DIR/sudoers"; then
  pass "sudoers names only the exact deploy script"
else
  fail "sudoers does not name exactly the deploy script"
fi
if grep -v '^#' "$RENDER_DIR/sudoers" | grep -q '\*'; then
  fail "sudoers contains a wildcard"
else
  pass "sudoers has no wildcard"
fi

# R2.9: the CI key is restricted and forced to the entry point.
if grep -Eq '^restrict,command="/usr/local/sbin/counterspell-deploy-entry" ssh-ed25519 ' "$RENDER_DIR/authorized_keys"; then
  pass "the deploy key line forces the entry point with restrict"
else
  fail "the deploy key line does not force the entry point with restrict"
fi

# R2.3: root login and passwords are off, only the two users may log in, MaxAuthTries is 3.
for line in 'PermitRootLogin no' 'PasswordAuthentication no' 'MaxAuthTries 3' 'AllowUsers brighton deploy'; do
  if grep -Fxq "$line" "$RENDER_DIR/sshd.conf"; then
    pass "sshd drop-in has: $line"
  else
    fail "sshd drop-in is missing: $line"
  fi
done

if ((failures > 0)); then
  echo "$failures check(s) failed" >&2
  exit 1
fi
echo "base role render checks passed"
