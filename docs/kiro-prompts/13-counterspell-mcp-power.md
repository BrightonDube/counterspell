# D.13 Spec 11: `counterspell-mcp-power` (Feature, Design-First)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Create a feature spec named counterspell-mcp-power (Design-First). Context: #[[file:docs/system-design.md]] §8 and the MCP/Powers plan in #[[file:docs/kiro-playbook.md]] A.5–A.6.
1) crates/mcp: an MCP server using the official Rust SDK (rmcp) exposing validate_pack, find_ghost_input, shrink_input, run_reference, explain_verdict, fit_complexity, reusing crates/engine (no duplicated logic). stdio transport. Every tool that executes code (run_reference, find_ghost_input, shrink_input) goes through the judge crate's sandbox runner and trusted comparator on a sandbox-only rootless Docker daemon, with the same images and limits as production; it must never spawn python/node directly on the developer's machine. Tool descriptions say so.
2) kiro-power/: a Kiro Power in the Agent Plugins format (plugin.json with keywords, mcp.json pointing at the server, skills/author-problem/SKILL.md and skills/hunt-ghost/SKILL.md).
3) Register the server in .kiro/settings/mcp.json with autoApprove only for read-only tools.
4) Kiro dogfooding demo (#[[file:docs/requirements.md]] MCP-3): a demo pack with a deliberately buggy reference or implementation; a scripted session where Kiro calls find_ghost_input, gets a minimized counterexample through the sandbox, fixes the bug, and re-verifies with validate_pack + find_ghost_input. Save the chat export and screenshots for the video and docs/KIRO_USAGE.md.
```
