# Requirements: pack-authoring-pipeline

Status: **approved for implementation** by Brighton on 15 Nov 2026 10:26. See `.kiro/steering/brighton-decisions.md` and `docs/kiro-log/pack-pipeline-blockers.md`. Paid services and spending stay held. Open questions keep their assumed choices except where those files record a decision.

## Introduction

Brighton does not write problems, reference solutions, hidden tests or hints himself. AI agents do. This spec defines the repeatable pipeline that turns one topic from the backlog into a complete Grimoire pack, checks it with a separate reviewer agent, and stops at a human gate. Brighton approves by merging a pull request. Only after that can the catalogue importer (`problem-catalogue`) load the pack.

The pipeline:

- runs in Kiro, using the repository's agent definitions, skills, prompt templates and hooks under `.kiro/`;
- uses the `counterspell-grimoire` validator for static checks, and the judge's sandbox through the `counterspell-mcp` code-running tools (or the judge CLI) for every check that runs code;
- writes each pack in two trees: a public tree in `packs/<slug>/` and a private tree in the private GitHub repository `BrightonDube/counterspell-packs-private` (suggested name; Brighton creates it);
- never imports, publishes or pushes anything by itself.

Sources: `docs/system-design.md` §4 (data model), §8 (ghost engine), §8b (efficiency familiar, complexity metadata); `docs/requirements.md` PCK-1 to PCK-3, MCP-1 to MCP-3, walkthrough W3, E9, E13; `docs/product-brief.md`; `docs/licence-rules.md`; `.kiro/steering/grimoire-authoring.md`; `.kiro/steering/brighton-decisions.md` (14 Nov 23:06 private-repo decision; 14 Nov 2026 "packs stay draft" decision; the "do not create the private repo" rule); `.kiro/specs/grimoire-packs/`; `.kiro/specs/ghost-engine/`; `.kiro/specs/counterspell-mcp-power/`; `.kiro/specs/hints/`; `.kiro/specs/efficiency-familiar/`; `.kiro/specs/problem-catalogue/`; `.kiro/specs/judge-sandbox/`; `docs/kiro-prompts/33-starter-pack-hardening-v2.md` (DRAFT, Brighton's quality bar for starter packs, read only); `docs/kiro-prompts/38-private-pack-split.md`.

### Out of scope

- Importing packs into Postgres, the catalogue API and `publish` (`problem-catalogue`).
- The judge's `PackRunner` implementation (`judge-sandbox` T20 to T22) and the grimoire `--sandbox` wiring (`grimoire-packs` T20). Sandbox stages wait for them.
- Moving existing pack files, creating the private GitHub repository, and rotating hidden cases. These are work for later tasks, after Brighton's approval (see PIP-35).
- Writing the seven new launch problems. This spec defines how they are produced. It does not produce them.
- Payments, community-submitted packs, and any paid model or hosting service.

## Glossary

- **Pipeline**: the ordered stages S0 to S13 that take one backlog topic to a reviewed draft pack.
- **Author agent**: the Kiro agent that drafts a pack, one stage at a time.
- **Reviewer agent**: a separate Kiro agent that scores a finished draft against a written checklist. It has no write access to pack files.
- **Orchestrator**: the batch script or the Kiro session that runs stages in order, enforces stop conditions and the loop cap, and writes state. It makes no judgement about quality.
- **Public tree**: `packs/<slug>/` in `BrightonDube/counterspell`. Holds `problem.md`, `meta.toml` (metadata and `[[samples]]` only), `hints.toml` and `originality.md`.
- **Private tree**: `packs/<slug>/` in `BrightonDube/counterspell-packs-private`. Holds `hidden.toml`, `generator.toml`, `reference/solution.py`, `incorrect/*.py`, `known_ghosts.toml`, `approval.toml`, and `reviews/`.
- **Assembled tree**: a temporary copy of the public and private trees of one slug, built under `target/pack-assembly/<slug>/`. Only the validator and the sandbox read it. It is deleted after the run, and it is never committed.
- **Content hash**: the SHA-256 of the canonical manifest of the assembled tree (`problem-catalogue` C06). Any change to any file changes it.
- **Hidden data**: `hidden.toml`, `generator.toml`, `reference/`, `incorrect/`, `known_ghosts.toml`, the retired-value list, and any expected output or ghost value taken from them.
- **Stop condition**: the check that must pass before a stage counts as done. Each stage in `design.md` lists its stop condition and diagnostic codes.
- **Review round**: one pass of the reviewer agent over one content hash.
- **Approval record**: `approval.toml` in the private tree. Brighton writes it. It names the content hash that was approved.
- **Launch set**: the ten polished launch problems in `docs/requirements.md` (F2): the three starter packs and seven new ones from `docs/pack-backlog.md`.

## Requirements

### A. Scope, roles and safety of the process

1. **PIP-1** THE SYSTEM SHALL leave every pack it produces with `status = "draft"`. No agent, script or hook in this pipeline SHALL set `status = "published"` or create or edit `approval.toml`.
2. **PIP-2** THE SYSTEM SHALL use four roles: the author agent, the reviewer agent, the orchestrator, and Brighton. The reviewer agent SHALL run in a separate session, SHALL receive the finished pack and the checklist but not the author's working notes, and SHALL have no tool that writes pack files.
3. **PIP-3** THE SYSTEM SHALL produce only original problems. Statements, titles, examples and trade dress SHALL come from the author's own wording (PCK-3, GRM-49). Authoring agents SHALL NOT open third-party problem statements while drafting.
4. **PIP-4** THE SYSTEM SHALL provide a batch mode that drives the pipeline for the whole launch set (ten packs) and leaves each pack at `draft` with a review summary ready for Brighton.
5. **PIP-5** THE SYSTEM SHALL use only tools already in the repository or free: Kiro, git, cargo, bash, and the GitHub free features that the repository already uses. It SHALL NOT call a paid model, paid review service or paid hosting. Any change to that rule is an open question for Brighton (design open question 1).

### B. Stages

6. **PIP-6** THE SYSTEM SHALL run the stages S0 to S13 in order, as defined in `design.md`. Each stage SHALL name its inputs, outputs and stop condition. A stage whose stop condition fails SHALL halt the pack, record the diagnostic code, and leave later stages not started.
7. **PIP-7** (S0 topic and concept selection) WHEN a topic is started, THE SYSTEM SHALL check that its slug is not already used in `packs/` or in the private tree, that its track and concepts are listed in `packs/catalogue.toml` (CAT-T2), that its difficulty is `easy`, `medium` or `hard`, and that a backlog entry exists for it.
8. **PIP-8** (S1 originality guard) THE SYSTEM SHALL write `originality.md` in the public tree. It SHALL record the idea in one paragraph, the reasoning for why it is original, which general patterns it uses (patterns only, never wording), and the checklist answers. IF the record is missing, or if a text marker from the originality list appears in any public or private file, THEN THE SYSTEM SHALL refuse the pack with `ORG-02` or `ORG-01` respectively.
9. **PIP-9** (S2 statement and examples) THE SYSTEM SHALL write `problem.md` that passes GRM-19 and GRM-20. Its `## Examples` section SHALL contain a call to the pack's function with the result of sample 1 and two to four lines of explanation (STM-03). Its constraints SHALL match `input_schema` exactly, and it SHALL say "integer" wherever negatives are allowed.
10. **PIP-10** (S3 signature, schema and metadata) THE SYSTEM SHALL write `meta.toml` that passes GRM-8 to GRM-18, with `origin = "original"`, `licence = "MIT"`, `pack_version = 1` for a new pack, one to five concept tags, and `complexity_note` of 240 characters or fewer. It SHALL NOT declare `reference_complexity` until a `[bench]` table exists (GRM-16, SBX-08).
11. **PIP-11** (S4 reference) THE SYSTEM SHALL write `reference/solution.py` in the private tree. It SHALL define the top-level function named in `meta.toml` (GRM-34). Its outputs SHALL be the only source of expected values for hidden tests, subject to the sandbox check in S10.
12. **PIP-12** (S5 hidden tests and generator) THE SYSTEM SHALL write `hidden.toml` in the private tree with at least 8 curated cases. The cases SHALL cover empty or minimal input, a single item, the maximum length, boundary values, negatives where allowed, duplicates or equal values, and the input of each known ghost. Each case SHALL carry a hand-derived `expected` and a `why` note of 1 to 120 characters. No case SHALL duplicate a sample, a case in the retired-value list (PIP-35), or another case. THE SYSTEM SHALL write `generator.toml` in the private tree with `seed` and `hidden_count` (5 to 50). The checks are HID-01 to HID-04 (grimoire-packs amendment X2).
13. **PIP-13** (S6 incorrect programs and known ghosts) THE SYSTEM SHALL write at least two incorrect programs in `incorrect/` and a matching `known_ghosts.toml` in the private tree. Each ghost SHALL describe a realistic beginner mistake (for example an off-by-one bound, a wrong starting value, forgetting the last element, mixing up index and value, or mishandling empty input). Each ghost's `got` SHALL be hand-traced and recorded in the author's notes. No ghost SHALL rely on a contrived bug such as calling `abs()` on a total. Each explanation SHALL follow GRM-31 and SHALL NOT claim that its input is the smallest.
14. **PIP-14** (S7 hints) THE SYSTEM SHALL write `hints.toml` with exactly four rungs (GRM-26). Each rung SHALL be general, SHALL NOT assume the learner's specific bug, and SHALL follow the ladder meanings in `hints` R1 to R4 (R1 nudge, R2 approach, R3 pseudo-code, R4 near-solution step). No rung SHALL contain a sample expected value, a hidden expected value, or a known-ghost `got` value (HNT-03). No fenced code block SHALL be longer than 3 lines (GRM-27).
15. **PIP-15** (S8 difficulty and complexity) THE SYSTEM SHALL set `difficulty` and a `complexity_note` that states the time and extra space of the reference and gives one short reason. It SHALL NOT add `[bench]` or `reference_complexity` in this pipeline. Those are added by the efficiency-familiar work (efficiency-familiar T40, version bump, X7).
16. **PIP-16** (S9 static validation) THE SYSTEM SHALL assemble the public and private trees (PIP-31) and run `counterspell-grimoire validate --deny-warnings` on the assembled tree. The stage passes only with zero errors and zero warnings.
17. **PIP-17** (S10 sandbox validation) THE SYSTEM SHALL run the reference on every sample (SBX-01), every hidden case and generated input (SBX-02, SBX-09), and every known ghost (SBX-03) through the judge's sandbox `PackRunner` only. A pack SHALL NOT pass S10 without a recorded sandbox image digest. Until that runner exists, the pack's state stays `static_complete` and cannot reach S13.
18. **PIP-18** (S11 ghost checks) THE SYSTEM SHALL run each incorrect program through `find_ghost_input` and require a verified counterexample within the configured budget that reproduces the same kind as its `known_ghosts.toml` entry (GHC-01 and GRM-43). It SHALL run the reference through the same tool and require "not found" within budget. Both calls SHALL use the judge's sandbox (MCP-1, MCP-2 of the counterspell-mcp Power, SEC-1), and the output SHALL NOT claim minimality (GHO-6, DEMO-4).
19. **PIP-19** (S12 review loop) THE SYSTEM SHALL run the reviewer agent on each content hash and record one scored report per round. The author SHALL address or rebut every finding, and the pack SHALL return to S0 to S9 for any change that the findings require.
20. **PIP-20** (S13 human gate) THE SYSTEM SHALL stop at the human gate and open a pull request for Brighton. The pack SHALL remain `draft` until Brighton merges it (section D).

### C. Reviewer agent and scoring

21. **PIP-21** THE SYSTEM SHALL keep a written reviewer checklist at `.kiro/pipeline/checklists/reviewer.md`. The checklist SHALL cover seven categories: correctness, clarity, originality, difficulty calibration, hint quality, ghost coverage, and accessibility of wording. Each category SHALL have at least one checklist item with a stable ID (`RV-<category>-<n>`).
22. **PIP-22** THE SYSTEM SHALL require the reviewer to score each category from 1 to 5 and to cite evidence for each score as a file and a pointer (for example `hints.toml rung 3`). Evidence SHALL NOT quote any hidden input or expected value. Hidden references use the pointer form `hidden.toml /case/3`.
23. **PIP-23** THE SYSTEM SHALL pass a review round only when all of these hold: every category scores 3 or more; the mean is 4 or more; correctness scores 5; originality scores 4 or more and has no ORG finding; and no finding has severity `blocker`.
24. **PIP-24** THE SYSTEM SHALL record each finding with an ID, a severity (`blocker`, `major`, `minor`), a category, and a file pointer. The author SHALL answer each finding as `fixed` (with the commit or content hash) or `rebutted` (with a reason). THE SYSTEM SHALL NOT accept an answer that is empty.
25. **PIP-25** THE SYSTEM SHALL cap review at three rounds per pack. IF a pack has not passed after round three, THEN THE SYSTEM SHALL set its state to `escalated`, write a blocker entry for Brighton in `docs/kiro-log/pack-pipeline-blockers.md`, and start no further rounds for that pack.
26. **PIP-26** WHEN the content hash changes after a review round, THE SYSTEM SHALL mark that round stale. The pack SHALL NOT reach S13 on a stale review. A new round SHALL run on the new hash.

### D. Human gate and publication

27. **PIP-27** THE SYSTEM SHALL produce, for each pack, two review outputs. The first is a one-page summary written for a non-expert reader, in `docs/pack-reviews/<slug>.md`. It states what the problem asks, why it is a good test, what the ghost shows, what could still be wrong, the scores in plain words, and the decisions Brighton must make. It SHALL contain no hidden data, no hidden case pointers and no reference source. The second is the full scored report, in the private repository under `reviews/<slug>/`.
28. **PIP-28** WHEN Brighton approves a pack, THE SYSTEM SHALL require that approval happens through a merged pull request. Brighton SHALL set `status = "published"` in the public `meta.toml` in a commit whose message contains "Approved by Brighton". He SHALL also write `approval.toml` in the private tree, naming the content hash, `approved_by`, `approved_at` and the public commit.
29. **PIP-29** THE SYSTEM SHALL NOT import, publish or announce any pack automatically. The importer SHALL refuse a pack whose `status` is `draft` (CAT-I3) and SHALL refuse a pack whose current content hash does not equal `approval.toml` (PIP-36).
30. **PIP-30** WHEN an approved pack is edited, THE SYSTEM SHALL set `status` back to `draft` in the same change, and the edit SHALL go through review and approval again. A content change also requires a larger `pack_version` (D11 of `problem-catalogue`).

### E. Public and private split

31. **PIP-31** THE SYSTEM SHALL keep the public tree (`problem.md`, `meta.toml` with samples only, `hints.toml`, `originality.md`) in the public repository, and the private tree (`hidden.toml`, `generator.toml`, `reference/`, `incorrect/`, `known_ghosts.toml`, `approval.toml`, `reviews/`) in the private repository. It SHALL NOT put a private file in the public tree, not even temporarily in a tracked path.
32. **PIP-32** THE SYSTEM SHALL NOT write hidden data to any public location. That covers public packs, `docs/`, `.kiro/pipeline/state/`, logs, PR descriptions, commit messages, digests, MCP results, and test output. Diagnostics SHALL give a pointer or a count only (GRM-25).
33. **PIP-33** THE SYSTEM SHALL read the private repository with a read-only deploy key or a fine-grained token supplied at run time, from an environment variable or a file outside the repository. The credential SHALL NOT appear in prompts, committed files, images, logs, error text or the assembled tree. Agents SHALL NOT read `.env` or any credential file (existing guard-read-secrets hook).
34. **PIP-34** IF the private tree for a slug is missing, unreadable or refused, THEN THE SYSTEM SHALL stop that pack at S0 with a clear code. It SHALL NOT continue on the public tree alone.
35. **PIP-35** THE SYSTEM SHALL rotate the hidden data of the three starter packs before any starter pack can be approved. The old values are already in public git history (`docs/kiro-prompts/33-starter-pack-hardening-v2.md`). Rotation SHALL write new hidden cases, a new generator seed, and a new `pack_version`. It SHALL record a SHA-256 of each retired canonical input and expected value in the private `retired-values.toml`. The hashes are checked by HID-03 and never shown.
36. **PIP-36** THE importer contract SHALL be: read both trees, join them by slug, assemble the combined manifest, require `approval.toml` to name that manifest hash, and refuse with a stable code otherwise. The importer SHALL NOT write any part of a refused pack (CAT-I11). The contract is implemented in `problem-catalogue` (amendment X3).

### F. Safety of generated code

37. **PIP-37** THE SYSTEM SHALL treat every reference, incorrect program, generator and hidden case as untrusted until it has run inside the judge's sandbox. No pack code SHALL run on the developer host, in the orchestrator, or in a reviewer session.
38. **PIP-38** THE SYSTEM SHALL route every code-running check through the judge sandbox, using the counterspell-mcp code-running tools (`run_reference`, `find_ghost_input`, `shrink_input`) or the judge CLI. No agent SHALL call `python`, `python3`, `node` or a shell on a pack file. The existing dangerous-shell guard SHALL block such commands on paths under `packs/` and in the assembled tree (PIP-42).
39. **PIP-39** WHILE the pipeline runs headless (batch mode), THE SYSTEM SHALL NOT approve code-running tools automatically (counterspell-mcp CONFIG-3). Batch mode SHALL run the static stages and the review. The sandbox stages S10 and S11 SHALL be run by the operator on the host with Brighton's approval, and they SHALL record the sandbox image digest.
40. **PIP-40** THE SYSTEM SHALL NOT use keyword or API blocklists as a security control for pack code (GRM-55, E14). The sandbox is the boundary.
41. **PIP-41** THE SYSTEM SHALL keep prompt templates and checklists free of secrets, credentials and hidden values. A test SHALL scan them for secret-like strings. Retired values and hidden expected values SHALL NOT enter an author or reviewer prompt, except that a reviewer may see the private tree when it runs in the private-capable session that Brighton configured (open question 9).

### G. Repository artefacts

42. **PIP-42** THE SYSTEM SHALL add the following artefacts. Kiro agent definitions in `.kiro/agents/` (`pack-author.json`, `pack-reviewer.json`). Skills in `.kiro/skills/grimoire-pack-authoring/SKILL.md` and `.kiro/skills/pack-review/SKILL.md`. Stage prompt templates in `.kiro/pipeline/prompts/`. Checklists in `.kiro/pipeline/checklists/` (`author.md`, `reviewer.md`). Pipeline state in `.kiro/pipeline/state/<slug>.toml`. The review summary template in `.kiro/pipeline/templates/`. The backlog in `docs/pack-backlog.md`. Review outputs in `docs/pack-reviews/`. Scripts in `scripts/pack-assemble.sh`, `scripts/pack-review-digest.sh` and `scripts/pack-batch.sh`.
43. **PIP-43** THE SYSTEM SHALL register hooks in `.kiro/hooks/`: a publish guard (a PreToolUse hook that blocks an agent write setting `status = "published"` or touching `approval.toml`), and an extended `pack-validate` hook (PostFileSave on `packs/**`). Hooks SHALL be created with the `createHook` tool, not by hand.
44. **PIP-44** THE SYSTEM SHALL extend `scripts/pre-push.sh` so a push fails when a commit changes a pack's `status` to `published` and its message lacks "Approved by Brighton" (`PUB-01`).
45. **PIP-45** THE SYSTEM SHALL keep the state file in the public repository, containing the stage, status, round, content hash, validator and ghost-check codes, reviewer scores and timestamps. It SHALL contain no hidden value (PIP-32).

### H. Tests and evidence

46. **PIP-46** THE SYSTEM SHALL include negative fixture packs that must be rejected, each with its code: a copied-text marker (`ORG-01`), a missing originality record (`ORG-02`), fewer than two known ghosts or a ghost that does not reproduce (`GHC-01`), a reference that fails a sample or hidden case (`SBX-01` / `SBX-09`), a hint that leaks a hidden or ghost value (`HNT-03`), a publish without approval (`PUB-01`), and a stale review (PIP-26).
47. **PIP-47** THE SYSTEM SHALL include a completeness test for the checklist: every reviewer category has at least one item, every item has a unique ID, and every ID referenced in a stage prompt exists in a checklist.
48. **PIP-48** THE SYSTEM SHALL include a dry run of the whole pipeline on one sample topic, `lantern-streaks` (a new original problem about the longest run of increasing lantern brightness readings), using a fake `PackRunner` in a temporary directory. The dry run SHALL reach S9 with a pass, S10 and S11 as `pending_judge`, and SHALL leave the repository unchanged.
49. **PIP-49** THE SYSTEM SHALL include tests for the loop cap and escalation (PIP-25) and for the batch orchestrator's refusal to set `published` (PIP-1, PIP-39). The tests SHALL NOT call a network, the GitHub API or a model. Model output is judged only by the validators and the checklist completeness test, never by unit tests of prose.
50. **PIP-50** WHEN a Rust change is made for this pipeline, THE SYSTEM SHALL meet the workspace Rust gate (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) and record a Rust lesson (`.kiro/steering/rust-teaching.md`).

## Traceability

| Area | Requirements | Built by |
|---|---|---|
| Stages S0 to S13 | PIP-6 to PIP-20 | design §3, prompts, grimoire codes |
| Reviewer and loop | PIP-21 to PIP-26 | design §6 and §7, `pipeline` module in grimoire |
| Human gate | PIP-27 to PIP-30 | design §8, `pre-push.sh`, publish guard |
| Split and importer contract | PIP-31 to PIP-36 | design §4, `problem-catalogue` X3 |
| Safety | PIP-37 to PIP-41 | design §9, hooks, sandbox |
| Artefacts | PIP-42 to PIP-45 | design §10 |
| Tests | PIP-46 to PIP-50 | design §12 |
