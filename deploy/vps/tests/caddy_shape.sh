#!/usr/bin/env bash
# Shape test for the Caddy edge (deploy-ops T08, R6.1 to R6.6). Checks the Caddyfile text and
# validates it with the pinned Caddy image, using only the local image (no pull).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CADDYFILE="$ROOT/Caddyfile"
CADDY_IMAGE="${CADDY_IMAGE:-caddy:2.10.2-alpine}"
failures=0

fail() { echo "FAIL: $*" >&2; failures=$((failures + 1)); }
pass() { echo "ok: $*"; }

# Requires the text to appear in the Caddyfile.
expect() {
  if grep -Fq -- "$1" "$CADDYFILE"; then pass "$2"; else fail "$2 (missing: $1)"; fi
}

# Compression stays on the normal routes.
# shellcheck disable=SC2016 # the braces are Caddy's placeholder syntax, not a shell expansion
expect '{$CADDY_SITE_ADDRESS::80}' "site address comes from CADDY_SITE_ADDRESS with a :80 default"

# R6.6: readiness is internal only. Caddy answers 404 before any proxy rule.
expect 'respond /api/ready 404' "/api/ready answers 404 at the edge"
ready_line="$(grep -nF 'respond /api/ready 404' "$CADDYFILE" | head -1 | cut -d: -f1 || true)"
proxy_line="$(grep -nF 'reverse_proxy' "$CADDYFILE" | head -1 | cut -d: -f1 || true)"
if [[ -n "$ready_line" && -n "$proxy_line" && "$ready_line" -lt "$proxy_line" ]]; then
  pass "the readiness 404 comes before the first reverse_proxy"
else
  fail "the readiness 404 must come before the first reverse_proxy"
fi

# R6.4: the run-event stream is unbuffered.
expect 'flush_interval -1' "run-event SSE route is unbuffered"
expect '^/api/runs/[^/]+/events$' "run-event SSE route matches only the events path"

# R6.2 and R6.3: headers on every response, and no Server header.
expect 'Strict-Transport-Security' "HSTS header"
expect 'X-Content-Type-Options "nosniff"' "nosniff header"
expect 'Referrer-Policy' "Referrer-Policy header"
expect 'Permissions-Policy' "Permissions-Policy header"
expect 'X-Frame-Options "DENY"' "X-Frame-Options header"
expect 'Content-Security-Policy' "Content-Security-Policy header (report-only until Q17 is decided)"
expect '-Server' "Server header is removed"

# Compression stays on the normal routes.
expect 'encode zstd gzip' "compression is kept for normal routes"

if ((failures > 0)); then
  echo "$failures check(s) failed" >&2
  exit 1
fi

# Validate with the pinned Caddy image, once for each site address form.
if ! command -v docker > /dev/null 2>&1; then
  echo "FAIL: docker is needed to validate the Caddyfile" >&2
  exit 1
fi
for address in ':80' 'counterspell.example'; do
  if docker run --rm --pull=never \
    -v "$CADDYFILE:/etc/caddy/Caddyfile:ro" \
    -e "CADDY_SITE_ADDRESS=$address" \
    "$CADDY_IMAGE" caddy validate --config /etc/caddy/Caddyfile > /dev/null 2>&1; then
    pass "caddy validate passes with CADDY_SITE_ADDRESS=$address"
  else
    fail "caddy validate fails with CADDY_SITE_ADDRESS=$address"
  fi
done

if ((failures > 0)); then
  echo "$failures check(s) failed" >&2
  exit 1
fi
echo "all Caddy edge checks passed"
