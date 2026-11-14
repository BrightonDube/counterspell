# 38 - Amend problem-catalogue for the private-pack split

Added 14 Nov 2026 23:06 after Brighton changed the hidden-data decision. This is a spec-only amendment of the already-written `problem-catalogue` spec. Do not implement any task. Approved by Brighton (14 Nov 2026).

Follow AGENTS.md and every file in `.kiro/steering/` (read `brighton-decisions.md` first, especially the 14 Nov 23:06 private-repo decision). Do not push, do not change branches, and do not start another Kiro session.

## Why this session exists

Session 34 already wrote `.kiro/specs/problem-catalogue/` under the earlier decision that hidden tests could stay in the public repo (see design.md open question 18). That decision was replaced at 23:06. Do not rewrite the spec from scratch. Amend `requirements.md`, `design.md` and `tasks.md` in place so they match the new decision. Do not change any file outside `.kiro/specs/problem-catalogue/`.

## Required split

- Hidden tests (`hidden.toml`, and generator seeds if they reveal hidden cases), reference solutions and ghost/incorrect solutions live in a private GitHub repo, not the public Counterspell repo.
- Suggested private repo name: `BrightonDube/counterspell-packs-private`. Do not create that repo. Do not move existing pack files.
- The public repo keeps `problem.md`, `meta.toml` (samples), hints and other public parts. `packs/catalogue.toml` stays public.
- The importer reads both trees, joins them by slug, validates the combined pack, and loads Postgres. Hidden parts stay server-only (judge role). They never appear in the public API, logs or SSE events.
- Access to the private repo is a deploy key or a token supplied at import time, stored outside the public repo. Never bake it into the public repo or images, never commit it, never log it. A missing or refused private tree is a failed import, not a silent public-only import.
- Hidden parts already in public git history must be rotated (new hidden cases) in the later pack work; this amendment only specifies the layout and importer. Note that rotation in the design so implementers do not just copy the old files.

Read `docs/kiro-prompts/33-starter-pack-hardening-v2.md` for the layout (it is a DRAFT; use it for the split, do not edit it). If `.kiro/specs/pack-authoring-pipeline/` already exists, stay consistent with it and do not edit it.

## Finish

Replace the open-question-18 assumption ("repo stays public for now") with this decision. Add or adjust requirements, the data-flow diagram, tasks (including credential handling and a refused public-only import test) and any SUPERVISED markers that touch secrets or hidden data. Keep the existing style (EARS IDs, waves, 30-minute tasks, named files). Commit the amended spec files only. Then stop. Do not implement.
