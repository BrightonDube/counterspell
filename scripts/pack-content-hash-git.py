#!/usr/bin/env python3
"""Canonical manifest content hash over git blob bytes.

Mirrors catalogue-cli read_tree + judge build_manifest, but reads blobs via
`git show` so hashes match what an importer sees on a fresh checkout (eol=lf).
Usage: pack-content-hash-git.py <pub-repo> <pub-ref> <pub-prefix> <prv-repo> <prv-ref> <prv-prefix> <slug>
"""
import hashlib
import subprocess
import sys

PACK_DIRS = ("reference", "incorrect")


def blob(repo, ref, path):
    return subprocess.run(
        ["git", "-C", repo, "show", f"{ref}:{path}"],
        capture_output=True, check=True,
    ).stdout


def ls_tree(repo, ref, prefix):
    out = subprocess.run(
        ["git", "-C", repo, "ls-tree", "-r", "--name-only", ref, "--", prefix],
        capture_output=True, check=True, text=True,
    ).stdout.split()
    files = {}
    for path in out:
        rel = path[len(prefix):].lstrip("/")
        parts = rel.split("/")
        if len(parts) == 1 or (len(parts) == 2 and parts[0] in PACK_DIRS):
            files[rel] = blob(repo, ref, path)
    return files


def main():
    pub_repo, pub_ref, pub_pre, prv_repo, prv_ref, prv_pre, slug = sys.argv[1:8]
    entries = {}
    for prefix, repo, ref, pre in (
        ("public", pub_repo, pub_ref, f"{pub_pre}{slug}"),
        ("private", prv_repo, prv_ref, f"{prv_pre}{slug}"),
    ):
        for rel, data in ls_tree(repo, ref, pre).items():
            if rel == "approval.toml":
                continue
            entries[f"{prefix}/{rel}"] = hashlib.sha256(data).hexdigest()
    canon = b"".join(
        f"{p}\0{entries[p]}\n".encode() for p in sorted(entries)
    )
    print(hashlib.sha256(canon).hexdigest())


if __name__ == "__main__":
    main()
