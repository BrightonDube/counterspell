#!/usr/bin/env bash
# Assembles the public and private trees of one pack into a temporary copy, then writes a
# manifest of SHA-256 hashes (pack-authoring-pipeline PA07; PIP-31, PIP-32, PIP-34; design D3).
#
# Usage: scripts/pack-assemble.sh <slug> <public-root> <private-root>
#
# Output: on stdout, the content hash of the manifest (one line). The manifest is written to
# $PACK_ASSEMBLY_ROOT/<slug>.manifest (default target/pack-assembly) with one "hash  path" line
# per file, sorted by path. This script never prints a file's contents, only names and codes.
#
# Exit codes: 0 assembled, 2 usage or bad slug, 3 refused (LAY-05 or symbolic link),
# 4 private or public tree missing (PIP-34), 5 a copy or hash step failed.
set -euo pipefail

readonly PRIVATE_FILE_NAMES=(hidden.toml generator.toml known_ghosts.toml approval.toml retired-values.toml)
readonly PRIVATE_DIR_NAMES=(reference incorrect reviews)

die() {
    local code="$1"
    shift
    printf 'pack-assemble: %s\n' "$*" >&2
    exit "$code"
}

if [[ $# -ne 3 ]]; then
    die 2 "usage: pack-assemble.sh <slug> <public-root> <private-root>"
fi
slug="$1"
public_root="$2"
private_root="$3"

# A slug is a directory name. Anything else (dots, slashes) could leave the packs folder.
if [[ ! "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    die 2 "the slug must be lowercase letters, digits and hyphens"
fi

public_dir="$public_root/packs/$slug"
private_dir="$private_root/packs/$slug"
assembly_root="${PACK_ASSEMBLY_ROOT:-target/pack-assembly}"
assembled="$assembly_root/$slug"
manifest="$assembly_root/$slug.manifest"

if [[ ! -d "$public_dir" ]]; then
    die 4 "the public tree for $slug is missing"
fi
# PIP-34: a missing or unreadable private tree stops the pack. The public tree alone is never used.
if [[ ! -d "$private_dir" || ! -r "$private_dir" ]]; then
    die 4 "PIP-34: the private tree for $slug is missing or unreadable"
fi

# A symbolic link could point outside the tree, so neither tree may contain one.
for tree in "$public_dir" "$private_dir"; do
    if [[ -n "$(find "$tree" -type l -print -quit)" ]]; then
        die 3 "a symbolic link was found in a pack tree; refusing"
    fi
done

# LAY-05: a private file must not sit in the public tree (checked at any depth).
for name in "${PRIVATE_FILE_NAMES[@]}"; do
    if [[ -n "$(find "$public_dir" -type f -name "$name" -print -quit)" ]]; then
        die 3 "LAY-05: the private file $name is in the public tree"
    fi
done
for name in "${PRIVATE_DIR_NAMES[@]}"; do
    if [[ -n "$(find "$public_dir" -type d -name "$name" -print -quit)" ]]; then
        die 3 "LAY-05: the private folder $name is in the public tree"
    fi
done

# LAY-05: the same relative path must not exist in both trees. Overwriting one with the other is refused.
collisions="$(cd "$public_dir" && find . -type f | LC_ALL=C sort)"
private_files="$(cd "$private_dir" && find . -type f | LC_ALL=C sort)"
shared="$(LC_ALL=C comm -12 <(printf '%s\n' "$collisions") <(printf '%s\n' "$private_files") || true)"
if [[ -n "$shared" ]]; then
    die 3 "LAY-05: a path exists in both trees; refusing to overwrite"
fi

# Rebuild the assembled copy from scratch. The slug was checked, so this stays under $assembly_root.
rm -rf -- "$assembled"
mkdir -p -- "$assembled"
if ! cp -R -- "$public_dir/." "$assembled/" || ! cp -R -- "$private_dir/." "$assembled/"; then
    die 5 "copying the trees for $slug failed"
fi

# The manifest: every file's SHA-256, sorted by path. Its own hash is the content hash.
if ! (cd "$assembled" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum) > "$manifest"; then
    die 5 "hashing the assembled tree for $slug failed"
fi
content_sha256="$(sha256sum -- "$manifest" | cut -d' ' -f1)"
printf '%s\n' "$content_sha256"
