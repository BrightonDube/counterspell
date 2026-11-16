#!/usr/bin/env bash
# Scans pack prompts, agents, skills, summaries and pack scripts for secret-like text and for
# retired-value markers (pack-authoring-pipeline PA19; PIP-33, PIP-41, PIP-32).
#
# Usage: scripts/pack-secret-scan.sh <file-or-folder>...
#
# For each hit it prints "<file>:<line> <code>" to stderr and exits 3. It never prints the
# matched line or the matched text, so the report cannot leak what it found. Exit 0 means clean.
#
# Codes: SEC-GH token, SEC-KEY API key or private key, SEC-PW password assignment,
# SEC-ENV path to a .env file, SEC-ECHO a script that echoes a credential variable,
# RET-01 a retired-value marker.
set -euo pipefail

if [[ $# -lt 1 ]]; then
    printf '%s\n' 'usage: pack-secret-scan.sh <file-or-folder>...' >&2
    exit 2
fi

# This scanner is not scanned: it has to name the patterns it looks for.
readonly SELF="pack-secret-scan.sh"

readonly -a CODES=(SEC-GH SEC-KEY SEC-PW SEC-ENV SEC-ECHO RET-01)
readonly -a REGEXES=(
    'gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}'
    '(^|[^A-Za-z0-9_])(sk|gsk|re)_[A-Za-z0-9_-]{20,}|PRIVATE_KEY|BEGIN [A-Z ]*PRIVATE KEY'
    '[Pp][Aa][Ss][Ss][Ww][Oo][Rr][Dd][[:space:]]*='
    '(^|[ "'"'"'`(/])\.env([^A-Za-z0-9_.]|$)'
    '(echo|printf)[^#]*(PACKS_PRIVATE|DEPLOY_KEY|TOKEN)'
    'RETIRED-[A-Z0-9]'
)

found=0

# Scans one file. A hit prints the file, the line number and the code, never the line.
scan_file() {
    local file="$1" index line
    for index in "${!REGEXES[@]}"; do
        while IFS= read -r line; do
            printf '%s:%s %s\n' "$file" "$line" "${CODES[$index]}" >&2
            found=1
        done < <(grep -E -n -I -- "${REGEXES[$index]}" "$file" | cut -d: -f1 || true)
    done
}

for target in "$@"; do
    if [[ -d "$target" ]]; then
        while IFS= read -r -d '' file; do
            [[ "$(basename -- "$file")" == "$SELF" ]] && continue
            scan_file "$file"
        done < <(find "$target" -type f -print0)
    elif [[ -f "$target" ]]; then
        [[ "$(basename -- "$target")" == "$SELF" ]] && continue
        scan_file "$target"
    else
        printf 'pack-secret-scan: %s does not exist\n' "$target" >&2
        exit 2
    fi
done

if [[ "$found" -ne 0 ]]; then
    exit 3
fi
exit 0
