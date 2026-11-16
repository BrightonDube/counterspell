#!/usr/bin/env bash
# Pre-pulls the enabled sandbox images by digest on the sandbox daemon, then checks each local digest
# against the pinned one (deploy-ops T13, R3.6, R3.7). Images are never pulled at run time.
#
# Reads only enabled language sections of languages.toml. A reference without a digest is refused.
# Talks to the rootless sandbox socket only (SANDBOX_DOCKER_SOCKET), never to the main daemon.
set -euo pipefail

LANGUAGES_TOML="${LANGUAGES_TOML:-/usr/local/share/counterspell/languages.toml}"
SANDBOX_DOCKER_SOCKET="${SANDBOX_DOCKER_SOCKET:?SANDBOX_DOCKER_SOCKET is not set (the rootless sandbox socket)}"

# Prints "<image>" for each enabled language. Exits non-zero for an image without a digest.
enabled_images() {
  awk '
    /^\[/ { enabled = 0; image = ""; next }
    /^[[:space:]]*enabled[[:space:]]*=[[:space:]]*true/ { enabled = 1 }
    /^[[:space:]]*image[[:space:]]*=/ {
      line = $0
      sub(/^[^"]*"/, "", line)
      sub(/".*$/, "", line)
      image = line
      if (enabled) { print image }
    }
  ' "$LANGUAGES_TOML"
}

docker_sandbox() {
  docker -H "unix://$SANDBOX_DOCKER_SOCKET" "$@"
}

main() {
  local failures=0 image digest repo_digests
  local -a images
  mapfile -t images < <(enabled_images)

  for image in "${images[@]}"; do
    if [[ "$image" != *@sha256:* ]]; then
      echo "refused: $image has no digest (tags are never pulled)" >&2
      exit 1
    fi
  done

  for image in "${images[@]}"; do
    digest="${image##*@}"
    echo "pulling $image on the sandbox daemon"
    docker_sandbox pull "$image" > /dev/null
    repo_digests="$(docker_sandbox image inspect --format '{{json .RepoDigests}}' "$image")"
    if [[ "$repo_digests" != *"$digest"* ]]; then
      echo "digest mismatch for $image: the local image does not match the pinned digest" >&2
      failures=$((failures + 1))
    else
      echo "ok: $image matches its pinned digest"
    fi
  done

  if ((failures > 0)); then
    echo "$failures image(s) failed the digest check" >&2
    exit 1
  fi
}

main "$@"
