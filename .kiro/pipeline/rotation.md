# Rotation procedure for the three starter packs (plan only)

Status: a written plan. Nothing in this file has been run. Running it needs Brighton's approval of the rotation (PIP-35), and the private repository must hold the files first.

## Why rotation is needed

The starter packs `count-pairs`, `diagonal-total` and `longest-run-length` were published with their hidden parts in the public git history (their `reference/`, `incorrect/` and `known_ghosts.toml` files, and the `[generator]` table in `meta.toml`). Moving those files to the private repository is not enough. Anyone who read the public history already has the old values, so new hidden values are needed (PIP-35). All three packs stay `draft`.

## Packs in scope

| Slug | Function | Current public hidden parts | Current `pack_version` |
|---|---|---|---|
| `count-pairs` | `count_pairs` | `reference/solution.py`, `incorrect/empty-min.py`, `incorrect/same-index.py`, `known_ghosts.toml`, `[generator]` in `meta.toml` | 1 |
| `diagonal-total` | `diagonal_total` | `reference/solution.py`, `incorrect/abs-diagonal.py`, `incorrect/assumes-square.py`, `known_ghosts.toml`, `[generator]` in `meta.toml` | 1 |
| `longest-run-length` | `longest_run` | `reference/solution.py`, `incorrect/empty-first-char.py`, `incorrect/forgot-last-run.py`, `known_ghosts.toml`, `[generator]` in `meta.toml` | 1 |

## Steps (run once per pack, after Brighton approves)

1. **Record the retired values.** Write the canonical input and the canonical expected value of every old hidden case and every old ghost input into the private `retired-values.toml` as SHA-256 hashes. Canonical means JSON with sorted keys and no spaces. Hash it with `printf '%s' '<canonical text>' | sha256sum`. Store only the hash, never the value. The hashes are checked by HID-03 and must never be shown.
2. **Write new hidden data in the private tree.** Replace `hidden.toml` with at least eight new curated cases. Derive each expected value by hand (PIP-12). Do not copy any case from the old file, and do not copy any value listed in step 1.
3. **Write a new generator seed.** Create `generator.toml` in the private tree with a new `seed` and a `hidden_count` between 5 and 50. Remove the `[generator]` table from the public `meta.toml`.
4. **Bump the version.** Set `pack_version = 2` in the public `meta.toml`. Keep `status = "draft"`. A content change also needs a larger `pack_version` (`problem-catalogue` D11).
5. **Re-run the stages.** Run S5 to S13 again for the pack. S6 writes new known ghosts and their incorrect programs, because the old ghosts are public. S9 runs on the assembled tree. S10 and S11 run on the host with the operator's approval (PIP-39). S12 runs a fresh review round on the new content hash (the round counter restarts only for a new hash, as PIP-25 requires).
6. **Move the old private files out of the public tree.** Only after the new private files are in place, delete `reference/`, `incorrect/` and `known_ghosts.toml` from `packs/<slug>/` in a commit that says so. The public history still holds the old values, which is why step 1 comes first.
7. **Stop at the human gate.** The pack stays `draft`. Brighton approves it by the pull request and `approval.toml` (PIP-28). Nothing in this procedure sets `published`.

## Checks for a reviewer

- Each of the three packs has steps 1 to 7 done, or a note saying which step is still open.
- No retired value appears in any new file, in any prompt, or in any log (PIP-32, PIP-41).
- Each `pack_version` moved from 1 to 2 in the same change as the new hidden data.
- The ghost explanations describe realistic mistakes and never claim minimality (GHO-6).
