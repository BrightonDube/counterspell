# D.5 Spec 3: `grimoire-packs` (Quick Spec)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Quick spec: grimoire-packs. Implement the problem pack format from #[[file:docs/system-design.md]] (§4 data model problem fields, §8) as files under packs/<slug>/ (problem.md, meta.toml, reference solutions, generator/shrinker hooks into crates/engine) plus a `counterspell-grimoire validate <pack-dir>` command (crate crates/grimoire) and the first original starter problems toward the 10 polished launch problems (no HackerRank/LeetCode text). Every pack has a Python reference (authoritative for expected outputs), authored hints, and optional known-ghost seeds with an authored Bug Autopsy explanation. Follow .kiro/steering/grimoire-authoring.md.
```
