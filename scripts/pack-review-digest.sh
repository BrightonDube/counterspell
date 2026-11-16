#!/usr/bin/env bash
# Builds the reviewer's input for one pack and publishes its public summary
# (pack-authoring-pipeline PA15; PIP-27, PIP-32; design §10).
#
# Usage:
#   scripts/pack-review-digest.sh prepare <slug> <assembled-root> <reports-dir>
#       Copies the assembled pack, the reviewer checklist and the stage reports into
#       $PACK_REVIEW_ROOT/<slug>/input (default target/pack-review). That folder holds hidden
#       files, so it stays under target/, which git ignores. Prints only that path.
#
#   scripts/pack-review-digest.sh publish-summary <slug> <summary-file>
#       Scans the summary (SUM-01). If the scan is clean, copies it to
#       $PACK_REVIEWS_DIR/<slug>.md (default docs/pack-reviews). Otherwise writes nothing.
#
# Exit codes: 0 done, 2 usage or bad slug, 3 refused by SUM-01, 4 missing input.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$script_dir/.."
checklist="$repo_root/.kiro/pipeline/checklists/reviewer.md"

# Fixed tokens that must never appear in a public summary (SUM-01). Names only, never values.
readonly FORBIDDEN_TOKENS=(
    "hidden.toml"
    "generator.toml"
    "known_ghosts.toml"
    "approval.toml"
    "retired-values.toml"
    "reference/"
    "incorrect/"
    "/case/"
    "CANARY-"
)

die() {
    local code="$1"
    shift
    printf 'pack-review-digest: %s\n' "$*" >&2
    exit "$code"
}

check_slug() {
    local slug="$1"
    if [[ ! "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
        die 2 "the slug must be lowercase letters, digits and hyphens"
    fi
}

prepare() {
    if [[ $# -ne 3 ]]; then
        die 2 "usage: pack-review-digest.sh prepare <slug> <assembled-root> <reports-dir>"
    fi
    local slug="$1" assembled_root="$2" reports_dir="$3"
    check_slug "$slug"
    local review_root="${PACK_REVIEW_ROOT:-target/pack-review}"
    local source="$assembled_root/$slug"
    local dest="$review_root/$slug/input"

    if [[ ! -d "$source" ]]; then
        die 4 "the assembled tree for $slug is missing; run pack-assemble.sh first"
    fi
    if [[ ! -d "$reports_dir" ]]; then
        die 4 "the reports folder for $slug is missing"
    fi
    if [[ ! -f "$checklist" ]]; then
        die 4 "the reviewer checklist is missing"
    fi

    # The slug was checked, so this path stays under $review_root.
    rm -rf -- "$dest"
    mkdir -p -- "$dest/$slug" "$dest/reports"
    cp -R -- "$source/." "$dest/$slug/"
    cp -R -- "$reports_dir/." "$dest/reports/"
    cp -- "$checklist" "$dest/checklist.md"
    printf '%s\n' "$dest"
}

publish_summary() {
    if [[ $# -ne 2 ]]; then
        die 2 "usage: pack-review-digest.sh publish-summary <slug> <summary-file>"
    fi
    local slug="$1" summary="$2"
    check_slug "$slug"
    if [[ ! -f "$summary" ]]; then
        die 4 "the summary file is missing"
    fi

    # grep -F matches fixed text. Only the token is reported, never the line that contains it.
    local token
    for token in "${FORBIDDEN_TOKENS[@]}"; do
        if grep -F -q -- "$token" "$summary"; then
            die 3 "SUM-01: the summary names a private file, a pointer or a canary; not published"
        fi
    done

    local reviews_dir="${PACK_REVIEWS_DIR:-docs/pack-reviews}"
    mkdir -p -- "$reviews_dir"
    cp -- "$summary" "$reviews_dir/$slug.md"
}

if [[ $# -lt 1 ]]; then
    die 2 "usage: pack-review-digest.sh prepare|publish-summary ..."
fi
command="$1"
shift
case "$command" in
    prepare) prepare "$@" ;;
    publish-summary) publish_summary "$@" ;;
    *) die 2 "unknown command; use prepare or publish-summary" ;;
esac
