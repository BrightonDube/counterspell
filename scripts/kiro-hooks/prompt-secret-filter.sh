#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
prompt=${USER_PROMPT:-}
input="${event}"$'\n'"${prompt}"
if grep -Eiq 'sk-[A-Za-z0-9]{20,}|gsk_[A-Za-z0-9_-]{20,}|re_[A-Za-z0-9_-]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{20,}|AIza[0-9A-Za-z_-]{20,}' <<< "$input"; then
  printf '%s\n' 'blocked: probable API key detected in the submitted prompt'
  exit 2
fi
exit 0
