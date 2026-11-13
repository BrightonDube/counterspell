#!/usr/bin/env bash
# Fails if any tracked (or, with --staged, staged) file looks like a secret/credential file.
# .env.example / *.example / *.sample templates are allowed.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
if [ "${1:-}" = "--staged" ]; then
  files=$(git diff --cached --name-only --diff-filter=ACMR)
else
  files=$(git ls-files)
fi
pattern='(^|/)(\.env(\..*)?|.*\.(pem|key|p12|pfx|jks|keystore|kdbx|ovpn|tfstate|tfstate\.backup|tfvars)|id_(rsa|dsa|ecdsa|ed25519)(\.pub)?|\.npmrc|\.pypirc|\.netrc|\.git-credentials|credentials(\.json)?|service[-_]account.*\.json|secrets?\.(ya?ml|json|toml)|.*\.age|\.htpasswd)$'
allow='(\.example|\.sample|\.template)$|(^|/)\.env\.example$'
bad=$(printf '%s\n' "$files" | grep -E -i "$pattern" | grep -E -v -i "$allow" || true)
if [ -n "$bad" ]; then
  echo "::error::Sensitive-looking files are tracked. Remove them (git rm --cached), add to .gitignore, and rotate anything real:"
  printf '  %s\n' $bad
  exit 1
fi
echo "No forbidden files tracked."
