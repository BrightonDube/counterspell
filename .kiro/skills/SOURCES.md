# Third-party Kiro skills (vendored, reviewed 2026-11-12)

Kiro CLI loads workspace skills from `.kiro/skills/<name>/SKILL.md` automatically (default agent, CLI >= 1.26; https://kiro.dev/docs/skills/).
Every file below was read before vendoring. None makes network calls or asks for secrets. Each folder carries its upstream LICENSE.

| Skill | Source | Commit | Licence |
|---|---|---|---|
| test-driven-development | https://github.com/obra/superpowers/tree/main/skills/test-driven-development | 22f0b463e7a5e579904dd6211a520321035526f7 | MIT (c) 2025 Jesse Vincent |
| systematic-debugging | https://github.com/obra/superpowers/tree/main/skills/systematic-debugging | 22f0b463e7a5e579904dd6211a520321035526f7 | MIT (c) 2025 Jesse Vincent |
| verification-before-completion | https://github.com/obra/superpowers/tree/main/skills/verification-before-completion | 22f0b463e7a5e579904dd6211a520321035526f7 | MIT (c) 2025 Jesse Vincent |
| requesting-code-review | https://github.com/obra/superpowers/tree/main/skills/requesting-code-review | 22f0b463e7a5e579904dd6211a520321035526f7 | MIT (c) 2025 Jesse Vincent |
| receiving-code-review | https://github.com/obra/superpowers/tree/main/skills/receiving-code-review | 22f0b463e7a5e579904dd6211a520321035526f7 | MIT (c) 2025 Jesse Vincent |
| svelte-core-bestpractices | https://github.com/sveltejs/ai-tools/tree/main/tools/skills/svelte-core-bestpractices | d3e755be770277d8c75fe61109b103b801b12775 | MIT (c) 2026 Svelte Contributors |

Rejected: `sveltejs/ai-tools` `svelte-code-writer` (runs `npx @sveltejs/mcp`, a network call); superpowers `using-git-worktrees`, `subagent-driven-development`, `finishing-a-development-branch` (branch/worktree changes conflict with "stay on brighton/kiro-setup"). No Rust/Axum skill from a reputable publisher was found; Rust rules stay in `.kiro/steering/`.

Project overrides (non-interactive Kiro runs): where a skill says "ask your human partner", write the question to `docs/kiro-log/<spec>-blockers.md` and continue with independent work.
