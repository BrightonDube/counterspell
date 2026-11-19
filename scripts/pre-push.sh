#!/usr/bin/env bash
# Local pre-push gate. Used by scripts/git-hooks/pre-push (core.hooksPath).
# Mirrors .github/workflows/ci.yml and security.yml as closely as a laptop
# allows, so failures are caught here instead of in CI:
#
#   gitleaks (pushed range)        forbidden-files        cargo fmt --check
#   pack-publish-check             prettier --check       eslint
#   svelte-check                   cargo clippy           cargo test --workspace
#   ignored pg_/valkey_ tests*     release build+testkit* judge sandbox build*
#   vitest                         vite build             npm audit*
#   terraform fmt/validate/tflint* cargo deny*
#
# Steps marked * are "extras": skipped by PRE_PUSH_FAST=1 or when their tool or
# services are missing (a note prints what was skipped and why). Everything
# else is mandatory. PRE_PUSH_SKIP=name1,name2 skips individual steps by name —
# for emergencies only; CI will still run them.
#
# Deliberately CI-only: Docker image builds, CodeQL, Semgrep, TruffleHog,
# OSV-Scanner, dependency-review, actionlint/zizmor and llvm-cov (coverage has
# no failing threshold yet). Any failure aborts the push (exit 1).
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

zero=0000000000000000000000000000000000000000
refs="$(cat)"
FAST="${PRE_PUSH_FAST:-0}"

skipped=""
note_skip() { skipped="$skipped  - $1 ($2)\n"; }

step() {
  # step <name> <command...>: run a mandatory check, honouring PRE_PUSH_SKIP.
  local name=$1; shift
  case ",${PRE_PUSH_SKIP:-}," in
    *",$name,"*) note_skip "$name" "PRE_PUSH_SKIP"; return 0 ;;
  esac
  echo "pre-push: $name"
  "$@"
}

extra() {
  # extra <name> <command...>: like step, but also skipped by PRE_PUSH_FAST=1.
  local name=$1; shift
  case ",${PRE_PUSH_SKIP:-}," in
    *",$name,"*) note_skip "$name" "PRE_PUSH_SKIP"; return 0 ;;
  esac
  if [ "$FAST" = "1" ]; then
    note_skip "$name" "PRE_PUSH_FAST"
    return 0
  fi
  echo "pre-push: $name"
  "$@"
}

# --- gitleaks (reads the ref list captured above) ---
GL=$(command -v gitleaks || true)
if [ -z "$GL" ] && [ -x "$HOME/.local/bin/gitleaks" ]; then
  GL="$HOME/.local/bin/gitleaks"
fi
pushing=0
if [ -n "$GL" ] && [ -x "$GL" ]; then
  cfg=()
  [ -f "$root/.gitleaks.toml" ] && cfg=(--config "$root/.gitleaks.toml")
  status=0
  while read -r _lref lsha _rref rsha; do
    [ "$lsha" = "$zero" ] && continue
    pushing=1
    if [ "$rsha" = "$zero" ]; then
      range="$lsha --not --remotes"
    else
      range="$rsha..$lsha"
    fi
    "$GL" git "${cfg[@]}" --log-opts="$range" --redact --no-banner --exit-code 1 "$root" || status=1
  done <<< "$refs"
  if [ "$status" -ne 0 ]; then
    echo "Push blocked: gitleaks found a possible secret. Rotate it, remove it, then push." >&2
    exit 1
  fi
else
  note_skip gitleaks "binary not found; CI will still scan"
  while read -r _lref lsha _rref _rsha; do
    [ "$lsha" = "$zero" ] && continue
    pushing=1
  done <<< "$refs"
fi

if [ "$pushing" -eq 0 ]; then
  exit 0
fi

# PUB-01 (PIP-44): a push may not publish a pack unless the commit says it was approved by Brighton.
while read -r _lref lsha _rref rsha; do
  [ "$lsha" = "$zero" ] && continue
  if [ "$rsha" = "$zero" ]; then
    publish_range=("$lsha" --not --remotes)
  else
    publish_range=("$rsha..$lsha")
  fi
  if ! "$root/scripts/pack-publish-check.sh" "${publish_range[@]}"; then
    echo "Push blocked: a pack was published without approval (PUB-01)." >&2
    exit 1
  fi
done <<< "$refs"

step forbidden-files bash scripts/check-forbidden-files.sh

step fmt cargo fmt --all -- --check

step prettier npm --prefix web run format:check

step eslint npm --prefix web run lint

step svelte-check npm --prefix web run check

step clippy cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# CI has no counterspell-packs-private checkout beside the repo. Pointing the
# override at a nonexistent path disables both the env root and the sibling
# fallback, so local tests can never pass on content CI cannot see.
step cargo-test env COUNTERSPELL_PACKS_PRIVATE_ROOT=/nonexistent cargo test --workspace --all-features --locked

# Ignored pg_/valkey_ tests need the throwaway services CI provides, and some of
# them TRUNCATE tables. They may only auto-run against a local service; a remote
# URL needs the explicit PRE_PUSH_DB_TESTS=1 opt-in.
url_is_local() {
  local rest=${1#*://} host
  rest=${rest##*@}
  host=${rest%%[:/]*}
  case "$host" in
    localhost|127.*|"") return 0 ;;
    *) return 1 ;;
  esac
}
if [ -n "${DATABASE_URL:-}" ] && [ -n "${VALKEY_URL:-}" ]; then
  if [ "${PRE_PUSH_DB_TESTS:-0}" = "1" ] || { url_is_local "$DATABASE_URL" && url_is_local "$VALKEY_URL"; }; then
    extra db-tests cargo test --workspace --all-features --locked -- --ignored pg_ valkey_
  else
    note_skip db-tests "URLs are not localhost; ignored tests TRUNCATE data — use throwaway services or PRE_PUSH_DB_TESTS=1"
  fi
else
  note_skip db-tests "DATABASE_URL/VALKEY_URL unset; ignored pg_/valkey_ tests run in CI"
fi

# Release builds must not pull in the testkit feature (mirrors the CI check).
release_no_testkit() {
  local log
  log=$(mktemp)
  cargo build --release --locked -p counterspell-server -p counterspell-judge -v 2>&1 | tee "$log"
  if grep -q 'feature="testkit"' "$log"; then
    echo "pre-push: the testkit feature reached a release build" >&2
    rm -f "$log"
    return 1
  fi
  rm -f "$log"
}
extra release-build release_no_testkit

# The judge binary built with only the sandbox feature, plus the identity
# guard refusing a rootful daemon (mirrors the sandbox CI job).
sandbox_feature() {
  cargo build -p counterspell-judge --features sandbox --locked
  local target bin
  target=$(cargo metadata --format-version 1 --no-deps --locked | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
  bin="$target/debug/counterspell-judge"
  if [ ! -x "$bin" ]; then
    echo "pre-push: $bin not found after build" >&2
    return 1
  fi
  if SANDBOX_DOCKER_SOCKET=/var/run/docker.sock "$bin" preflight; then
    echo "pre-push: preflight accepted a rootful Docker daemon" >&2
    return 1
  fi
}
extra sandbox-feature sandbox_feature

step vitest npm --prefix web run test

step web-build npm --prefix web run build

# Dependency advisories are reported but do not block: security.yml enforces
# them, and pre-existing findings on main would otherwise stop every push.
if command -v npm >/dev/null 2>&1; then
  extra npm-audit bash -c 'npm --prefix web audit --audit-level=high || echo "pre-push: WARNING npm audit found advisories (non-blocking here; security.yml gates them)"'
fi

# Terraform: fmt is always cheap; init+validate+tflint need the tools and may
# download providers, so they sit in the extras group.
if command -v terraform >/dev/null 2>&1 && find infra -name '*.tf' -print -quit 2>/dev/null | grep -q .; then
  step terraform-fmt terraform -chdir=infra fmt -check -recursive -diff
  tf_validate() {
    local env
    for env in infra/terraform/envs/*/; do
      [ -d "$env" ] || continue
      terraform -chdir="$env" init -backend=false -input=false
      terraform -chdir="$env" validate
    done
  }
  extra terraform-validate tf_validate
  if command -v tflint >/dev/null 2>&1; then
    extra tflint bash -c 'cd infra && tflint --init && tflint --recursive --format compact'
  else
    note_skip tflint "binary not found"
  fi
else
  note_skip terraform "terraform binary or infra/*.tf missing"
fi

if command -v cargo-deny >/dev/null 2>&1; then
  extra cargo-deny bash -c 'cargo deny --locked check advisories bans sources || echo "pre-push: WARNING cargo deny findings (non-blocking here; security.yml gates them)"'
else
  note_skip cargo-deny "binary not found; security.yml runs it"
fi

if [ -n "$skipped" ]; then
  printf 'pre-push: skipped steps:\n%b' "$skipped"
fi
echo "pre-push: all checks passed"
