#!/usr/bin/env bash
set -euo pipefail

event=$(cat)

if grep -Eiq -- '(^|[[:space:]])--privileged([[:space:]]|=|$)' <<< "$event"; then
  printf '%s\n' 'blocked: privileged containers are not allowed'
  exit 2
fi
if grep -Eiq -- '--network[=[:space:]]+host' <<< "$event"; then
  printf '%s\n' 'blocked: host networking is not allowed'
  exit 2
fi
if grep -Eiq -- 'seccomp[=:]unconfined' <<< "$event"; then
  printf '%s\n' 'blocked: unconfined seccomp is not allowed'
  exit 2
fi
if grep -Eiq -- '/var/run/docker\.sock' <<< "$event"; then
  printf '%s\n' 'blocked: the host Docker socket must not be mounted'
  exit 2
fi
if grep -Eiq -- 'rm[[:space:]]+-rf[[:space:]]+/(($|[[:space:]])|[^[:space:]])' <<< "$event"; then
  printf '%s\n' 'blocked: refusing recursive deletion from filesystem root'
  exit 2
fi
if grep -Eiq -- 'git[[:space:]]+push[^[:cntrl:]]+--force' <<< "$event"; then
  printf '%s\n' 'blocked: force-pushing is not allowed by this hook'
  exit 2
fi
if grep -Eiq -- 'curl[^[:cntrl:]]+[|][[:space:]]*(sh|bash)([[:space:]]|$)' <<< "$event"; then
  printf '%s\n' 'blocked: piping curl directly into a shell is not allowed'
  exit 2
fi

exit 0
