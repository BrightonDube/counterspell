# Starter pack v2 blockers (16 Nov 2026)

## Status
No commits were made for this brief. The working tree is back to its last committed state.

## What happened
1. I replaced the `abs-diagonal` ghost with `both-diagonals` (program, ghost entry and the model-runner vector). The first write to `packs/diagonal-total/known_ghosts.toml` was blocked by hook LAY-05: a private file must not be written under the public `packs/` folder.
2. I had already deleted `packs/diagonal-total/incorrect/abs-diagonal.py` and created `both-diagonals.py` under the public folder. Both edits broke the state of the repo, so I reverted them with `git checkout` and removed my new file. No hook was bypassed.

## Why this is blocked
The brief moves ghosts (`incorrect/`, `known_ghosts.toml`), `reference/` and `hidden.toml` to `BrightonDube/counterspell-packs-private`. LAY-05 confirms that these files may not live under public `packs/`. The public validator reads `packs/<slug>/` as one tree, so moving these files first would make `validate --all packs` fail. The combined-tree reader that Part 3 (`pack-import`, I02) describes does not exist yet.

## Decision needed from Brighton
Pick one order:
- (a) Build the combined public + private tree reader in `counterspell-grimoire` first (a pure change), then move the ghost, reference and hidden files to the private checkout (`~/code/counterspell-packs-private`), then do the `both-diagonals` swap there.
- (b) Keep ghosts in public for now and do the swap there. This contradicts LAY-05, so I have not taken it.

My assumed choice is (a). I have not started it, because it changes the validator's input contract.

## Work that does not depend on this
These are public parts and can proceed once Brighton confirms:
- Statement rewrites (`problem.md`) for the three starter packs.
- Hint rung 4 rewrites (`hints.toml`), and the GRM-26 label change in grimoire-packs `requirements.md`.
- `complexity_note` in each `meta.toml`.
- Hand trace for `both-diagonals` on `[[1]]`: the reference gives 1 (`grid[0][0]`, added once); the ghost program adds `grid[0][0] + grid[0][0]` = 2, so `got = 2`.

## Deferred (not started)
- Part 2 (curated `hidden.toml`, HID-01..04, SBX-09, STM-03) and the rotation of hidden cases.
- Part 3 (`pack-import` spec and implementation, I01 to I10).
- Part 4 (authoring skill, backlog, review digest, publish guard, seven new packs).
