# Spec 16: `pack-authoring-pipeline` (AI agents author and review problem packs)

Added 14 Nov 2026 after the spec-gap review. Brighton will not write problems, solutions or hints himself; AI agents do, and he only approves. Spec mode only.

```
Spec: pack-authoring-pipeline. Specify a repeatable pipeline in which AI agents (Kiro, using the repository's agents, skills and the counterspell-mcp tools) design original problems and produce complete Grimoire packs, a separate review agent checks them, and Brighton approves before anything is imported. Read #[[file:docs/system-design.md]] (§4, §8, §8b), #[[file:docs/requirements.md]] (PCK-1 to PCK-3, MCP-1 to MCP-3, walkthrough W3, the 10 polished launch problems), .kiro/steering/grimoire-authoring.md, .kiro/specs/grimoire-packs/ (format, validator codes, starter packs), .kiro/specs/ghost-engine/, .kiro/specs/counterspell-mcp-power/ (validate_pack, find_ghost_input, shrink_input, the Power), .kiro/specs/hints/ (authored ladder R1 to R4), .kiro/specs/efficiency-familiar/ (complexity metadata), docs/licence-rules.md, and the problem-catalogue spec if it exists. If docs/kiro-prompts/33-starter-pack-hardening*.md exist, read them as Brighton's quality bar for packs; do not edit them.

Cover:
1. Stages with explicit inputs, outputs and stop conditions: topic and concept selection against the catalogue plan; originality guard (no HackerRank/LeetCode text, titles or trade dress; record the reasoning); statement and examples; signature and input schema; reference solution; curated hidden tests (edge cases, boundaries, adversarial cases) plus generator settings; deliberately incorrect solutions and known ghosts with Bug Autopsy text; hint ladder R1 to R4; difficulty and complexity metadata; static validation, then sandbox validation and ghost checks through the judge-backed tools only.
2. A separate reviewer agent with a written checklist and a scored report (correctness, clarity, originality, difficulty calibration, hint quality, ghost coverage, accessibility of wording). Failing packs loop back with findings; a cap on loops.
3. The human gate: packs stay status = draft; Brighton approves by merging a pull request (and only then sets published), with a one-page review summary per pack written for a non-expert. Nothing is imported or published automatically.
4. Repository artefacts: Kiro agent definitions, skills, prompt templates and hooks under .kiro/, a pack review report location, and a batch mode to produce the 10 launch problems. Everything runs on the free tooling already in the repo; no paid service.
5. Safety: generated code (references, incorrect solutions, generators) is untrusted until it runs in the sandbox; no pack code runs on the host; secrets never enter prompts.
6. Tests: fixture packs that must be rejected (copied text markers, missing ghosts, reference failing its own tests, hints that leak the answer), checklist completeness, and a dry-run of the pipeline on one sample topic.
List any choice that affects cost, licensing or publishing as an open question for Brighton.
```
