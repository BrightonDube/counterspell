# Hints spec blockers and open decisions

Run: 16 Nov 2026 implement run (non-interactive). Each entry names the task, the reason, and what would unblock it.

## G2: authored hint length (decision needed from Brighton)

- The hints spec (G2, R1.1 and the migration design) says each rung is 1 to 1,200 characters.
- The database CHECK in `migrations/20261116100000_catalogue_core.sql` (`problem_hints.body`) and the grimoire-packs validator (`hints.toml` rule HNT-02, tested in `crates/grimoire/src/hints.rs`) both say 1 to 600 characters.
- The stricter 600 limit is kept, so no stored hint can break the database check. G2 stays open until Brighton picks one value. If 1,200 wins, the catalogue CHECK, the grimoire limit and the grimoire tests all change together, with a `test-change:` marker that cites the requirement.
- G2 also asks for errors that name the problem slug and the rung. The current messages name the file and the JSON pointer, not the slug. This is left open with G2.

## H11 / ghost read: `ghosts` table has no owner

- The hints store reads the submission's ghost (R3.1). No migration creates a per-submission `ghosts` table yet. The catalogue spec lists the owner as unresolved (decision D16, `.kiro/specs/problem-catalogue/tasks.md`).
- Unblock: decide the owner of `ghosts`. Until then the ghost read returns no ghost when the table is missing, so AI hints still work on the verdict and code (R3.5).
