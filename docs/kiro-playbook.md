# Counterspell × Kiro playbook: let Kiro write the specs; we steer
*Written Wed 11 Nov 2026, 23:30 SAST (UTC+2) by Brighton's Research Assistant. All Kiro facts come from kiro.dev docs pages opened 11 Nov 2026 (raw copies in `research/kiroween/raw/kiro/`) and the live Kiroween rules and resources pages. **[U]** = unverified (check in the IDE before relying on it); **[E]** = my estimate.*

> **Revision 11 Nov 2026, 23:55 SAST: Brighton's corrections.**
> - He **already redeemed** the 4,000-credit Kiroween grant and has used about 2,200. About **1,700 credits remain, and that is all.**
> - His account shows **every model** (GPT-5.6, Opus 5.5, Qwen and others), so all models are treated as available. The only constraint is credits, so model choice is planned by credit multiplier.
> - Two surfaces need a paid *subscription* per the official docs, so credits alone don't unlock them: **Kiro Web** and **headless CLI** (API key). Both are dropped (see A.8, A.9).
> - Boilerplate and UI polish move to other coding tools (see **Part F**, with the rules quote). Kiro credits go to what judges score as Kiro implementation: specs, PBT, hooks, MCP, powers and agents.

> **Revision Thu 12 Nov 2026, 01:45 SAST: Brighton's grill-me decisions on the ChatGPT critique** (`review/chatgpt-critique-2026-11-12.md`; full list in the system design's "Decision log 2026-11-12"). Marked **[D 12 Nov]** below.
> - **Kept:** all 14 specs and all 17 hooks, but the hook defects the critique found are fixed in A.4 (nonzero exits block PreToolUse/UserPromptSubmit, `set -o pipefail`, Svelte-safe SPDX headers, a non-blocking pre-task check, scoped ghost-hunt hint, changed-pack validation, a task log that records task, commit and checks).
> - **Rewritten prompts** D.4, D.6, D.7 and D.9 now follow the critique's §5.4–5.7 versions, adjusted to Brighton's decisions: a rootless sandbox-only Docker daemon, worker-owned verdicts, minimized counterexamples, a Postgres job queue with `run_events` (Valkey for cache and rate limits only), Python + JavaScript at launch (Java and C++ stretch), shrink animation, Bug Autopsy and the Kiro dogfooding demo.
> - Spec 5 is renamed **`run-lifecycle`** (it no longer uses Valkey Streams). One canonical Cargo layout (`counterspell-{server,judge,engine,grimoire,mcp,common}`).
> - **Build order: landing and visuals first** (Brighton's choice), with a **hard checkpoint on Wed 25 Nov**: a real fail → ghost flow on the VPS, or visual work pauses (Part C).

**Brighton's direction (11 Nov):** *"I would rather Kiro be the one that builds the specs… you can give the prompts, but don't write the specs… tell him the brief… the system design. Let him be the one who designs the spec files… plan the hooks, plan the everything."*

**What changed:**
- The 14 hand-written specs are retired (moved to `design/_superseded-handwritten-specs/`). The old steering and hooks are now reference only.
- We give Kiro five brief files (`docs/product-brief.md`, `docs/requirements.md`, `docs/system-design.md`, `docs/palette.md`, `docs/licence-rules.md`) plus the prompts in Part D.
- Kiro generates the steering, specs, tasks and code. We write hook JSON, custom-agent and MCP config only where the docs require a config file and Kiro can't generate it better. Even then, we can ask Kiro to create them from the plan in Part A.

---

## 0. Why this matters for judging
The rules (live page, 11 Nov) score **Technological Implementation (20%)**, verbatim:
> "How thoroughly and skillfully does the project use Kiro? Was there a thoughtful strategy to AI development? Was a variety of Kiro tools used impactfully? Does the project show a deep understanding of Kiro features? Was there experimentation?"

The submission write-up should cover:
- Kiro IDE / CLI / Web / Crew (and how they differed);
- custom agents;
- agent hooks;
- spec-driven vs vibe coding;
- steering strategy;
- MCP;
- property-based testing;
- powers.

So the plan below deliberately touches **every** one of these and produces evidence for each one (Part A.10).

## 0.1 Kiro feature facts this plan relies on (verified 11 Nov 2026)
| Feature | What the docs say (short) | Surfaces | Source (kiro.dev/docs/…) |
|---|---|---|---|
| Feature specs | `requirements.md` (EARS) → `design.md` → `tasks.md` in `.kiro/specs/<name>/`. Two variants: **Requirements-First** or **Design-First** (High-Level or Low-Level design). Multiple specs per repo are recommended. | IDE, CLI, Web | specs/, specs/feature-specs/ |
| Bugfix specs | `bugfix.md` with Current / Expected / **Unchanged** behaviour; tasks include property tests that prove the fix and guard against regressions. | IDE, CLI, Web | specs/bugfix-specs/ |
| Quick Spec | All three files generated without approval gates (asks questions up front). | IDE, CLI | custom-agents/built-in |
| Analyze Requirements | Streams clarifying questions and gaps on a requirements doc (takes minutes). | IDE, CLI | specs/ |
| Correctness / PBT | Kiro pulls testable properties out of the EARS requirements during design, generates property-based tests and **shrinks counterexamples**. Optional per spec. | IDE | specs/correctness/ |
| Run all tasks | Executes tasks in parallel waves along a dependency graph. "Sync Files" marks tasks that are already done. | IDE | specs/ |
| Steering | `.kiro/steering/*.md` (workspace) and `~/.kiro/steering/` (global). The **Generate Steering Docs** button creates `product.md`, `tech.md` and `structure.md`. Inclusion modes: `always`, `fileMatch` (+`fileMatchPattern`), `manual` (`#name` or `/name`), `auto` (needs `name` + `description`). File refs: `#[[file:path]]`, `#[[file:path:12-28]]`, `#[[folder:dir]]`. **AGENTS.md** is supported at the root and in subdirectories (always included). | IDE, CLI, Web | steering/ |
| Hooks | `.kiro/hooks/<id>.json`, schema `v1`. Triggers: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostFileCreate`, `PostFileSave`, `PostFileDelete`, `PreTaskExec`, `PostTaskExec`, `Stop` (+ PreCompact / PostCompact). Actions: `command` (shell; **no credits**) or `agent` (prompt; **consumes credits**). File triggers fire **only on agent edits**. Non-zero exit blocks PreToolUse and UserPromptSubmit (IDE). `{{filePath}}` works in commands, and STDIN carries JSON context. **Hooks don't run on Web.** | IDE, CLI V3 | hooks/, hooks/types, hooks/actions, ide/whats-new-v1/hooks |
| Permission modes | Supervised, **Accept file changes** (the old "Autopilot") and **Approve for me** (a model judges unruled actions). Plus `permissions.yaml` rules. | IDE | (autonomy pages) |
| Checkpoints | One per prompt. Restore rewinds files and context. They don't track MCP or bash side-effects. CLI has rewind/fork. | IDE, CLI | chat/checkpoints |
| Plan mode | Read-only exploration that produces a plan, then hands off to the default agent. | IDE, CLI | custom-agents/built-in |
| Custom agents & subagents | `.kiro/agents/<name>.json|.md` with name, description, tools, resources (`file://`, `skill://`), permissions, prompt, model, mcpServers and hooks. Any custom agent can run as a **subagent**, automatically (by description) or explicitly, **in parallel** (with a DAG). On Web, `.kiro/agents/` agents are subagents too. | IDE, CLI, Web | custom-agents/, custom-agents/subagents |
| Skills | `.kiro/skills/<name>/SKILL.md`, auto-loaded or used as a slash command. | IDE, CLI, Web | skills/ |
| Powers | One-click bundles of MCP + skills (Agent Plugins format: `plugin.json` + `skills/` + optional `mcp.json`; legacy `POWER.md` still works). Install from the registry or GitHub; create in the IDE or CLI v3 ("Build a Power"). The registry includes Context7, Snyk, Aikido, SonarQube, Postman, Neon, Supabase, Figma and more. | IDE, CLI v3, Web (install) | powers/, kiro.dev/powers |
| MCP | `.kiro/settings/mcp.json` (workspace) or `~/.kiro/settings/mcp.json`. Local (`command`/`args`/`env`/`autoApprove`/`disabledTools`) or remote (`url`, `headers`, OAuth handled by Kiro). Tool IDs look like `mcp_<server>_<tool>`. | IDE, CLI, Web | mcp/ |
| Context providers | `#codebase #file #folder #git diff #terminal #problems #url #repository #current #steering #docs #spec #mcp` | IDE | chat/context |
| Workflows (new) | Declarative multi-agent graphs, recipes in `.kiro/workflows/`, run in the background. Opt-in. | IDE, CLI, Web | workflows/ |
| Memory | Cross-session memory, local per user. | IDE, CLI | memory/ |
| Local command sandbox | Off by default (IDE 1.3+). | IDE | (sandbox page) |
| CLI V3 | Unified harness, custom agents, `/goal` loop (default 5 iterations), ACP (JetBrains / Zed). **Headless:** `kiro-cli chat --v3 --no-interactive --trust-all-tools "…"` with `KIRO_API_KEY`. **API keys need a Pro-or-higher *subscription* (docs), so this is dropped.** The interactive CLI works. | CLI | cli/v3, cli/headless, cli/chat/goal, cli/acp |
| Kiro Web | Default / Spec / Autonomous modes. Opens PRs. Learns from PR comments. **Needs a paid subscription (docs), so this is dropped.** | Web | web/ |
| Crew | Open-source local agent (`kirocrew` gateway on localhost:5476). App Kit apps use an `app.json` manifest (agents, skills, MCP, cron, UI). Has its own "Best Kiro Crew App" prize. | local | crew/ |
| Chat export | Right-click a chat tab → **Export Chat** gives a zip (`session.json`, `messages.jsonl`, sub-executions). Perfect for evidence. | IDE | chat/ |
| Models (credit multiplier vs Auto = 1.0x) | GPT-5.6 Sol 4.4x / Terra 2.2x / **Luna 0.6x**. Claude Opus 5.5 2.0x, Opus 5/4.8/4.7/4.6/4.5 2.2x, **Sonnet 5.5 1.3x**, Sonnet 5/4.6/4.5/4.0 1.3x, **Haiku 5.5 0.06x** (≤100K tokens; 0.3x above), Haiku 4.5 0.4x. Grok 4.7 2.7x, Kimi K3 2.4x, GLM-5 0.5x, **DeepSeek 3.2 0.25x**, MiniMax M2.5 0.25x, MiniMax M2.1 0.15x, **Qwen3 Coder Next 0.05x**. GPT-5.6 doubles above 272K tokens (Luna 1.2x). The docs note that the same multiplier can still cost different credits (tokenizer and thinking depth). | all | models/ (re-checked 23:45 SAST) |

---

# PART A: Feature-by-feature plan

## A.1 Steering: let Kiro generate it, then add a few targeted files
**Step 1, seed.** Copy `kiro-brief/*.md` into the repo's `docs/`. Add a root `AGENTS.md` that is only 6 lines long and points to `docs/` (AGENTS.md is always included on every surface, including Web and CLI).

**Step 2, generate.** In the IDE Kiro panel, click **Generate Steering Docs**. It writes `product.md`, `tech.md` and `structure.md`. Then paste prompt **D.1**, which tells Kiro to rewrite those three files from our `docs/` brief using `#[[file:…]]` references (so the brief stays the single source of truth) and to create the extra files below with the right front-matter.

**Step 3, review.** Brighton reads each file with the Markdown Review button, comments, and has Kiro revise.

| Steering file (Kiro writes it) | Inclusion | Why |
|---|---|---|
| `product.md` | `always` | What Counterspell is, the users and the MVP. References `#[[file:docs/product-brief.md]]`. |
| `tech.md` | `always` | The fixed stack, the "don't change without asking" list (including the Postgres job queue, Valkey for cache and rate limits only, and the rootless sandbox-only Docker daemon [D 12 Nov]), the licence allow-list via `#[[file:docs/licence-rules.md]]`. |
| `structure.md` | `always` | Workspace layout (`crates/{server,judge,engine,grimoire,mcp,common}` as `counterspell-<dir>` packages, `web/`, `packs/`, `languages/languages.toml`, `deploy/`): the one canonical layout from system design §2 [D 12 Nov]. |
| `rust-teaching.md` | `always` | Brighton is new to Rust. Every Rust change ends with a 3–5 line "Rust lesson". No `unwrap()` outside tests. Explain borrow errors in plain English. |
| `security.md` | `always` | No secrets in code, `.env` never read or written by the agent, ownership checks on every query, OAuth state + PKCE. |
| `sandbox.md` | `fileMatch: ["crates/judge/**", "languages/**", "deploy/**"]` | Docker flags, limits, network none, seccomp, verdict mapping (system design §6–7). The trusted worker owns tests, expected outputs, comparator, per-test deadlines and verdicts; sandboxes only on the rootless sandbox daemon; never run untrusted code outside the sandbox, including from MCP [D 12 Nov]. |
| `frontend.md` | `fileMatch: ["web/**"]` | Svelte 5 runes, Bits UI, Tailwind tokens from `#[[file:docs/palette.md]]`, GSAP only on the landing page and the familiar, reduced motion. |
| `grimoire-authoring.md` | `fileMatch: ["packs/**"]` | The problem pack format, generators, shrinkers, original problems only. |
| `ghost-engine.md` | `fileMatch: ["crates/engine/**"]` | Determinism (seeded RNG), shrink invariants (every accepted step strictly reduces a size metric), proptest conventions. Say "minimized counterexample", never "smallest possible" [D 12 Nov]. |
| `api-conventions.md` | `auto` (name + description) | JSON error shape, cookie sessions, SSE event names. Loaded when Kiro decides it's relevant. |
| `kiro-usage-log.md` | `manual` (`/kiro-usage-log`) | On demand: "append what we just did with Kiro to `docs/KIRO_USAGE.md`". This replaces the missing Manual hook trigger. |
| `release-checklist.md` | `manual` (`/release-checklist`) | Security, accessibility and demo-account checklist for freeze day. |
| `crates/judge/AGENTS.md` | always, subdir | A reminder that the judge is the security boundary, for any surface. |

## A.2 Spec plan: Kiro writes every spec; we only kick it off
Rules for us:
- One spec per feature. Kick each off with its Part D prompt, which points at the system-design section with `#[[file:docs/system-design.md]]` and says what's fixed.
- Brighton reviews requirements → design → tasks at the approval gates.
- **Design-First** where the architecture is already decided. **Requirements-First** where behaviour is the hard part. **Quick Spec** for small, low-risk UI.
- **Bugfix spec** for any real bug that takes over 15 minutes. Aim for at least 2 during the build; they make great write-up material.
- Turn on **Correctness / PBT** on specs that mark it (IDE only).
- **Run all tasks** only on Quick Spec / UI specs. Core specs are run task-by-task and supervised.

| # | Spec (folder name) | Type | Analyze Req.? | PBT | Task mode | Model | When (SAST) |
|---|---|---|---|---|---|---|---|
| 1 | `repo-skeleton` | Feature, **Design-First (Low-Level)** | no | no | Accept file changes; Run all tasks | Luna (0.6x) | Thu 12 Nov |
| 2 | `ghost-engine` | Feature, **Requirements-First** | **yes** | **yes** | **Supervised**, one task at a time | **Sonnet 5.5** (1.3x) for requirements/design + PBT; Luna for simple tasks; Opus 5.5 only if stuck twice | Sun 15 Nov (spec), tasks from Tue 17 Nov |
| 3 | `grimoire-packs` | Quick Spec | no | light | Accept; Run all tasks | Haiku 5.5 (0.06x) | Tue 17 Nov |
| 4 | `judge-sandbox` | Feature, **Design-First (Low-Level)** | **yes** | **yes** (limits) | **Supervised** | **Sonnet 5.5**; one deep audit on **Opus 5.5** (2.0x) | Wed 18 Nov |
| 5 | `run-lifecycle` (was `runs-streams-sse`) [D 12 Nov] | Feature, Design-First (High-Level) | no | yes (claim exclusivity, event ordering, replay) | Supervised for the claim/lease and terminal write, Accept for the rest | Sonnet 5.5 for the design; Luna for tasks | Fri 20 Nov |
| 6 | `auth` | Feature, **Requirements-First** | **yes** | yes (tokens) | **Supervised** | Sonnet 5.5 | Tue 24 Nov |
| 7 | `ide-workspace` (the ghost workspace, main Costume Contest showcase) | Feature, **Requirements-First** [D 12 Nov, per critique §5.7] (Kiro writes the spec; **other tool implements** the `[UI]` tasks, see Part F) | no | no | Kiro implements the SSE client, ghost card, shrink animation and Bug Autopsy wiring | Luna for the spec; Haiku 5.5 / Qwen3 Coder Next for Kiro's tasks | Sat 14 Nov (spec + `[UI]` shell on fixtures); real-API wiring Mon 23 Nov |
| 8 | `language-drivers` (JavaScript Must; Java + C++ tasks marked `[stretch]`) [D 12 Nov] | Feature, Design-First | no | yes (driver round-trip) | Accept, supervised review | Luna; Haiku 5.5 for config | Thu 26 Nov (JS); Mon 30 Nov (stretch) |
| 9 | `hints` | Quick Spec | no | no | Accept | Haiku 5.5 | Thu 26 Nov |
| 10 | `dashboard-fsrs` (basic dashboard Must; FSRS tasks marked `[stretch]`) [D 12 Nov] | Feature, Requirements-First (scheduler + API in Kiro; charts/UI by other tool) | no | yes (scheduler) | Accept | Luna | Sat 28 Nov |
| 11 | `counterspell-mcp-power` (sandboxed execution + Kiro dogfooding demo) [D 12 Nov] | Feature, Design-First | no | no | Supervised | Sonnet 5.5 | Fri 27 Nov |
| 12 | `efficiency-familiar` (measured mood + AI speech bubble; live guess `[stretch]`) [D 12 Nov] | Feature, Requirements-First | no | **yes** (complexity fit) | Supervised for the fit in Kiro; SVG/GSAP animation polish by other tool (art may already exist from the visuals phase) | Sonnet 5.5 for the fit + PBT | Fri 27 – Sat 28 Nov |
| 13 | `landing-steel-night` | Quick Spec (Kiro spec; **other tool implements**) | no | no | — | Haiku 5.5 | **Fri 13 Nov (visuals first [D 12 Nov])** |
| 14 | `break-room` | Quick Spec (Kiro spec; **other tool implements**; stretch after the freeze-safe core; cut first) | no | no | — | Haiku 5.5 | Mon 30 Nov |
| — | `deploy-ops` | **Plan mode** (no spec): read-only plan, then execute | — | — | Supervised | Luna | Thu 12 Nov + Tue 1 Dec |
| — | `bugfix-*` (×2+) | **Bugfix spec** | — | yes (auto) | Supervised | Sonnet 5.5 | as bugs appear |

**Spec numbers are IDs, not build order.** The build order is in Part C.

**Why this split is good for judges:** it shows all four spec flavours (Req-First, Design-First, Quick, Bugfix), plus Plan mode for vibe-style ops. The write-up can contrast spec-driven vs vibe coding with real examples.

## A.3 Property-based testing (Kiro Correctness + our own engine)
This is our best story. **The ghost engine *is* a property-based tester for learners, and we build it with Kiro's property-based tester.**
- In the `ghost-engine`, `judge-sandbox`, `run-lifecycle`, `auth`, `language-drivers`, `dashboard-fsrs` and `efficiency-familiar` specs, enable Correctness during design. Ask Kiro to phrase acceptance criteria so the properties are extractable (prompts in Part D already ask for this).
- **Properties we expect Kiro to derive** (it writes them; this list is the checklist Brighton reviews against):
  - **Ghost engine:**
    1. Every returned ghost reproduces its claimed failure when re-run independently.
    2. Every accepted shrink step strictly reduces the size metric.
    3. The same seed gives the same ghost.
    4. Every generated input passes the problem's validator.
    5. Comparators are reflexive and tolerance-symmetric.
    6. The reference solution never fails its own generator.
    7. The search terminates within its candidate, evaluation and time budgets.
  - **Sandbox limits:**
    1. For any program in the hostile corpus (fork bomb, infinite loop, 2 GB alloc, socket connect, write outside /tmp, giant stdout, **printing fake verdict JSON** [D 12 Nov]), the verdict is in {TLE, MLE, RE, OLE, WA} and is never "Accepted".
    2. The container is always removed.
    3. Wall time ≤ limit + 1 s.
    4. There's no network for any destination.
  - **Run lifecycle [D 12 Nov]:** no two workers hold the same job; every job reaches exactly one terminal state; after a reconnect with `Last-Event-ID` the client receives every event with a higher `seq`, in order, and drops duplicates by `seq` (at-least-once delivery + dedupe, not "exactly once" on the wire).
  - **Auth:** magic-link tokens are single-use and expire. A session never reads another user's data.
  - **Efficiency familiar:** on synthetic timings drawn from n, n log n and n² (with noise), the fit picks the right class, or "unsure", never a wrong class with high confidence.
  - **FSRS:** the next interval is non-negative and grows after "good" answers.
- **Libraries:** `proptest` (MIT/Apache) in Rust and `fast-check` (MIT) for TypeScript heuristics.
- **Evidence:** screenshot Kiro's PBT panel showing a shrunk counterexample. Our product then shows the same idea to learners.

## A.4 Hooks plan (Kiro creates the JSON from this table; prefer `command` = free)
All files go in `.kiro/hooks/`, with scripts in `scripts/kiro-hooks/`. Hooks fire on **agent** edits only, in the IDE and CLI V3 (not Web).

**Fixed 12 Nov [D 12 Nov], from critique §5.2/§8.1:** all 17 hooks are kept. Every script starts with `#!/usr/bin/env bash` and `set -euo pipefail` (or, in POSIX `sh`, captures the real command's exit status before any pipe), so a failing `cargo test` can't hide behind `tail`. **Any nonzero exit blocks** PreToolUse and IDE UserPromptSubmit hooks (and PreTaskExec), so hooks that only inform print to stdout and exit 0.

| # | Hook | Trigger | Matcher | Action | What it does | Credit impact |
|---|---|---|---|---|---|---|
| 1 | `rust-fmt-on-save` | PostFileSave | `crates/.*\.rs$` | command: `rustfmt --edition 2021 {{filePath}}` | Keeps Rust formatted | **0** |
| 2 | `rust-check-on-save` | PostFileSave | `crates/.*\.rs$` | command: `scripts/kiro-hooks/rust-check.sh` (`set -o pipefail`; `cargo clippy -q --all-targets -- -D warnings`, last 40 lines of stderr; clippy's own non-zero exit is preserved and sends errors to the agent) | Catches compile and clippy errors right away; the agent fixes them in the same turn | 0 for the hook. The agent's fix uses normal turn credits. |
| 3 | `engine-tests-on-save` | PostFileSave | `crates/engine/.*\.rs$\|packs/.*` | command: `scripts/kiro-hooks/engine-tests.sh`: `set -o pipefail; cargo test -q -p counterspell-engine 2>&1 \| tail -40` so the exit status is `cargo test`'s, not `tail`'s [D 12 Nov] | **Test-on-save** for the core library, including proptests | 0 (+ the agent's fix) |
| 4 | `ghost-hunt-on-test-failure` | PostToolUse | `shell` | command: `scripts/kiro-hooks/ghost-hint.sh` reads STDIN JSON. **Scoped [D 12 Nov]:** only when the shell call was a pack or engine test (`cargo test -p counterspell-engine`, `-p counterspell-grimoire`, or a test named `pack_*`) and the output contains `FAILED` / `panicked`, it prints to stdout: *"A pack/engine test failed. Consider calling `@counterspell/find_ghost_input` (it runs through the sandbox) or reading the proptest shrunk case, and explain the minimized counterexample before editing."* Always exits 0 (a suggestion, not a block). Other test failures are left alone. | **Ghost-finder on test failure:** makes Kiro use our own MCP to minimise its own bugs (dog-fooding) | 0 for the hook. The MCP call happens inside the agent turn. |
| 5 | `guard-secrets` | PreToolUse | `write` | command: `scripts/kiro-hooks/guard-secrets.sh` exits **2** if the STDIN JSON targets `.env`, `*.pem`, `id_rsa*` or `secrets/` (`.env.example` allowed). It also blocks content matching `sk-[A-Za-z0-9]{20,}`, `gsk_…` or `re_…` (Resend). Exits 0 otherwise. | **Secrets guard:** blocks the write. A **supplementary** check, not a security boundary: keep real secrets outside the repo, use file permissions and Kiro's `permissions.yaml` [D 12 Nov] | 0 |
| 6 | `guard-read-secrets` | PreToolUse | `read` | same script in read mode | Stops the agent reading `.env` into context through the read tool. Supplementary: it can't stop every shell command from reading files | 0 |
| 7 | `guard-dangerous-shell` | PreToolUse | `shell` | command: exit 2 on `--privileged`, `--network host`, `seccomp=unconfined`, `/var/run/docker.sock` mounts into user containers, `rm -rf /`, `git push --force`, `curl … \| sh`; exit 0 otherwise | Sandbox safety while Kiro writes the judge. A supplementary blocklist (easy to bypass by accident); Kiro's local command sandbox is the stronger control once tested (Part E #12) | 0 |
| 8 | `licence-check-on-deps` | PostFileSave | `(Cargo\.toml\|package\.json)$` | command: `scripts/kiro-hooks/licences.sh`: `cargo deny check licenses` + `npx --yes license-checker-rseidelsohn --onlyAllow "<allow-list>" --excludePackages gsap` in `web/` [U: confirm that npm tool's licence and flags] | **Licence check** on every dependency change (rules require compliant licences) | 0 |
| 9 | `web-check-on-save` | PostFileSave | `web/src/.*\.(svelte\|ts)$` | command: `scripts/kiro-hooks/web-check.sh`: `set -o pipefail; cd web && npx svelte-check --threshold error --output human \| tail -30` (keeps svelte-check's exit status) | Type errors surface right away | 0 |
| 10 | `pack-validate` | PostFileSave | `packs/.*\.(toml\|json\|py\|rs)$` | command: `scripts/kiro-hooks/pack-validate.sh {{filePath}}`: validates **only the changed pack** with `cargo run -q -p counterspell-grimoire -- validate packs/<slug>` (crate name fixed [D 12 Nov]) | Problem packs stay valid | 0 |
| 11 | `spdx-header-on-create` | PostFileCreate | `\.(rs\|ts\|svelte)$` | command: add the header if missing, **with file-type syntax [D 12 Nov]**: `// SPDX-License-Identifier: MIT` for `.rs`/`.ts`, and `<!-- SPDX-License-Identifier: MIT -->` as the first line of `.svelte` files (never `//` in Svelte markup) | Licence hygiene | 0 |
| 12 | `pre-task-green` | PreTaskExec | — | command: `cargo check -q --workspace`, print the result (pass, or the first 30 error lines) to stdout, and **always exit 0** [D 12 Nov]. A nonzero exit would block the very task meant to repair the tree | Kiro starts each task knowing whether the tree is green | 0 |
| 13 | `task-done-changelog` | PostTaskExec | — | **command** (default): appends `date`, branch, the **spec and task ID** (from STDIN JSON [U field name]), the **commit hash** and the **verification result** (fmt/clippy/test pass or fail) to `docs/kiro-log/tasks.md`; a diff-stat alone is not evidence [D 12 Nov] · **agent** (optional variant): "Add one line to `docs/CHANGELOG.md` (Keep a Changelog) and one line to `docs/KIRO_USAGE.md` saying which spec/task finished, which hooks fired and anything surprising." | **Spec task completion → CHANGELOG/docs** | Command 0. Agent ≈ 0.2–0.6 credits per task on Auto [E] ≈ 25–60 over the build. **Recommendation:** use the command variant and summarise weekly with `/kiro-usage-log`. |
| 14 | `stop-session-log` | Stop | — | command: append timestamp, model (if in STDIN) [U], `git diff --stat` and the last commit to `docs/kiro-log/sessions.md`. Uses `confirm` with `confirmCommand` so it asks only when files changed. | **Agent-stop summary** for the "How we used Kiro" write-up | 0. An agent-type Stop hook would cost credits on *every* turn, so avoid it. |
| 15 | `session-start-context` | SessionStart | — | command: print today's milestone from `docs/plan.md` (create it from Part C on day 1; the script prints "no plan.md" and exits 0 if it's missing), `git log --oneline -3` and open bugfix specs, capped at 40 lines [D 12 Nov]. stdout goes into context. | Kiro always knows where we are | 0 (adds ~200 tokens per session) |
| 16 | `prompt-secret-filter` | UserPromptSubmit | — | command: exit 2 if `$USER_PROMPT` contains an API-key pattern (any nonzero blocks submission in the IDE; CLI prompt-submit behaviour differs [U], test it); exit 0 otherwise. Supplementary, not a guarantee | Brighton never pastes keys into chat | 0 |
| 17 | `precompact-snapshot` | PreCompact | — | command: copy the current spec task list and `git status` into stdout for the summariser | Long sessions keep their place | 0 |

**Notes:**
- **Blocking (fixed 12 Nov):** the shell-hook docs say a nonzero exit reports an error, and for PreToolUse and IDE UserPromptSubmit it **blocks** the invocation (the hooks/types page also lists exit 2 for PreTaskExec). So there is **no "exit 1 = warn"**: use exit **2** when we mean "block", and print to stdout with exit **0** when we only want to inform. CLI prompt-submit behaviour differs [U].
- **Exit status through pipes:** every script uses `set -o pipefail` (bash) or captures the real command's status before piping, so `cmd | tail` can't turn a failure into a pass.
- `{{filePath}}` is documented for file triggers. Tool payloads arrive as JSON on STDIN. Write payload field names (e.g. `path`) are [U], so the guard scripts grep the whole JSON.
- **Credits:** only #13 (agent variant) costs credits by itself. Everything else is free shell. Hooks that exit non-zero do make the agent spend its *current* turn fixing things, which is the point.

## A.5 MCP servers (`.kiro/settings/mcp.json`; checked on GitHub 11 Nov)
| Server | Status 11 Nov | Use | Config notes |
|---|---|---|---|
| **counterspell-mcp** (ours, Rust `rmcp`) | built in spec 11 | Tools `validate_pack`, `find_ghost_input`, `shrink_input`, `run_reference`, `explain_verdict`, `fit_complexity`. Kiro uses it while authoring problems and debugging, and in the **Kiro dogfooding demo** [D 12 Nov]. Suggested by hook #4. **Code-running tools execute through the judge crate's sandbox runner on a sandbox-only rootless Docker daemon, never directly on the dev machine** [D 12 Nov]. | local: `cargo run -q --release -p counterspell-mcp`. `autoApprove` read-only tools only (not the code-running ones). |
| **GitHub**: `github/github-mcp-server` (MIT, active) | ✅ | Issues for bugs → Bugfix specs; PRs; Actions logs | The Kiro docs example uses the Docker image `ghcr.io/github/github-mcp-server` with `GITHUB_PERSONAL_ACCESS_TOKEN` from env. A remote/OAuth option exists [U for Kiro]. Fine-grained PAT for this repo only. Keep `disabled: true` until needed. |
| **Postgres**: `crystaldba/postgres-mcp` "Postgres MCP Pro" (MIT, pushed Aug 2026) | ✅ (the official `@modelcontextprotocol/server-postgres` is **archived**) | Inspect the dev schema, explain slow queries, check migrations | **Dev DB only**, restricted/read-only access mode [U flag name]. A PreToolUse hook on `@postgres` blocks `DROP\|TRUNCATE\|DELETE`. |
| **Docs / fetch** | Kiro has built-in `web_search` / `web_fetch` on all surfaces, so a fetch MCP isn't needed. Add **Context7** (`upstash/context7`, MIT, active; also a Kiro Power) for version-accurate Axum / SvelteKit / bollard / Bits UI docs. | Up-to-date API docs | as a Power (one click) |
| **Playwright MCP** (`microsoft/playwright-mcp`, Apache-2.0, active) | ✅ optional | UI smoke tests and **automatic screenshots for Devpost** | local `npx @playwright/mcp@latest` |

Turn on MCP tool search if many tools are enabled, to save context. If the local command sandbox is on, check that stdio MCP servers still start [U].

## A.6 Powers
- **Install:**
  - **Context7** (docs);
  - **one** security scanner power for a freeze-day scan. **Aikido** or **Snyk** are in the registry; both may need a free account [U]. Pick one, or skip.
  - Optional: **Postman** for API tests.
- **Build:** a **Counterspell Power** (spec 11), in the Agent Plugins format:
  - `plugin.json` (name, description, keywords: "ghost input", "grimoire", "problem pack", "shrink");
  - `mcp.json` → counterspell-mcp;
  - `skills/author-problem/SKILL.md` (how to write a Grimoire problem with generator + shrinker);
  - `skills/hunt-ghost/SKILL.md` (minimise a failing input before fixing).
  - Publish it in the repo under `kiro-power/` and test it with **Add Custom Power → Import power from GitHub**.
  - **Judge story:** "anyone can install our power and author Counterspell problems with Kiro."
- [U] The powers marketing page says powers bundle "MCP tools, steering files, and hooks". The create docs list plugin.json, skills and mcp.json. Check in the Power Builder whether steering and hooks can be included.

## A.7 Custom agents and subagents (Kiro writes these from the brief; `.kiro/agents/*.md`)
| Agent | Tools | Prompt essence | Where |
|---|---|---|---|
| `rust-reviewer` | read, shell (allow `cargo clippy\|test\|check` only), no write | Review the diff for ownership mistakes, `unwrap`, blocking in async, error types and SQL injection. Teach one Rust lesson. Output a checklist. | IDE + CLI. Run as a **subagent** after each core spec. |
| `sandbox-auditor` | read, shell (allow `docker inspect`, `grep`), no write | Audit `crates/judge` + `languages.toml` against `#[[file:docs/system-design.md:302-325]]`-style §7 rules [check line numbers after copying], the hostile corpus and the PBT results. Output PASS/FAIL per rule. | IDE + CLI. Run on Mon 16 Nov, Wed 25 Nov and Wed 2 Dec. One deep run on **Opus 5.5 (2.0x)**; the others on Sonnet 5.5. |
| `problem-author` | read, write `packs/**`, `@counterspell` | Write original problems using the `author-problem` skill; validate with the MCP | IDE |
| `docs-scribe` | read, write `docs/**` | Turn `docs/kiro-log/*` into `docs/KIRO_USAGE.md` and README sections | IDE / CLI. Model: Haiku 5.5 (0.06x). |

- **Parallel review demo:** "Use rust-reviewer and sandbox-auditor in parallel on this branch." That's two subagents with separate context windows, which makes a good screenshot.
- **Workflows (experiment):** a `.kiro/workflows/ship-feature` recipe: implement (default) → rust-reviewer ∥ sandbox-auditor → docs-scribe. It's opt-in and new, so it's good for the "experimentation" criterion.

## A.8 CLI: headless in CI, ACP, /goal
- **Interactive CLI** (works on any plan): `kiro-cli` with `--agent rust-reviewer` for terminal reviews, and `/goal --max 5 "all engine proptests pass and clippy is clean"` for a self-verifying loop (good write-up material).
- **Headless in CI: dropped.** It needs `KIRO_API_KEY`, and kiro.dev/docs/cli/headless says "API key authentication is only available for Kiro Pro, Pro+, Pro Max, and Power subscribers". That's a subscription, not credits. CI stays plain `cargo` / `svelte-check` / `cargo deny`.
- **Interactive CLI instead** (it signs in like the IDE): run `rust-reviewer` from the terminal once per core spec, and run one `/goal` loop. That's enough to answer the "IDE vs CLI" write-up prompt.
- **ACP:** optional. Mention that Zed/JetBrains can drive the same agent; we don't need it.

## A.9 Kiro Web and Crew (stretch, write-up value)
- **Kiro Web: dropped.** kiro.dev/docs/web says "Ensure you have a paid Kiro subscription (Pro or higher). Kiro Web is not available on the free tier." The write-up contrasts IDE (specs, PBT, hooks), CLI (reviews, `/goal`) and, if attempted, Crew.
- **Crew stretch: "Night Shift Familiar".** A Crew App Kit app (`app.json`) with:
  - a cron job that sends Brighton the daily Counterspell problem and his FSRS reviews;
  - the counterspell-mcp tool to explain a ghost on request.
  The rules require a separate public GitHub repo for the Crew bonus. Start only after freeze, and only if everything else is green. **[U] Whether Crew needs a subscription, and how many credits its model calls use (the app manifest example uses `"model": "auto"`).** Budget 0. Only try it if more than 150 credits of reserve are left on Wed 2 Dec.

## A.10 Evidence capture (for the Devpost write-up and the <3 min video)
1. **Commit `.kiro/` always:** steering, specs (with their history in git), hooks, agents, settings/mcp.json (no secrets) and workflows. Judges can tour `/.kiro`.
2. **`docs/KIRO_USAGE.md`**, one section per write-up prompt in the rules (IDE / CLI / Web / Crew, custom agents, hooks, spec vs vibe, steering, MCP, PBT, powers), with links to files.
3. **Chat export** (right-click tab → Export Chat) for every spec kickoff and both bugfix specs. Save to `docs/kiro-log/chats/` with secrets scrubbed (keep the zips private if in doubt [U]).
4. **Screenshots** (Playwright MCP or manual) into `docs/media/kiro/`:
   - the spec approval gates;
   - the Analyze Requirements questions;
   - the PBT shrunk counterexample;
   - the hook firing (guard-secrets blocking `.env`);
   - parallel subagents;
   - the MCP `find_ghost_input` call;
   - the **Kiro dogfooding demo** [D 12 Nov]: Kiro calls `find_ghost_input` on a deliberately buggy reference or implementation, gets a minimized counterexample through the sandbox, fixes the bug, and re-verifies;
   - the Power install;
   - the CLI `/goal`;
   - checkpoint restore.
5. **Logs from hooks #13 and #14:** `docs/kiro-log/tasks.md` and `sessions.md` are the raw timeline.
6. **Video beat (20–30 s):** a split screen. Kiro's PBT shrinks a counterexample for *our* engine, then Counterspell shrinks a ghost for the *user* (the shrink animation). Then the Kiro dogfooding demo (find → fix → re-verify) [D 12 Nov], then a 5 s `/.kiro` tree tour.
7. **Metrics:** credits used per spec (from the Kiro usage panel; screenshot it each Sunday), task count, and hooks that blocked something.

---

# PART B: Credit budget (1,700 credits, and that is ALL)
**Facts:**
- Brighton has about **1,700** credits left from the redeemed grant, and no more are coming. All models are available.
- Multipliers (kiro.dev/docs/models, 23:45 SAST): Opus 5.5 2.0x, Sonnet 5.5 1.3x, Auto 1.0x, Luna 0.6x (1.2x above 272K tokens), GLM-5 0.5x, DeepSeek 3.2 0.25x, MiniMax M2.1 0.15x, **Haiku 5.5 0.06x** (≤100K tokens; 0.3x above), **Qwen3 Coder Next 0.05x**.
- Per-action costs aren't published. These are **[E]** estimates at 1.0x: spec generation ≈ 10–20; one core task ≈ 4–8; Analyze Requirements ≈ 8. **Measure after spec 1 and re-cut.**

**Model routing (by multiplier):**
| Work | Model | Why |
|---|---|---|
| Requirements/design for hard specs (ghost engine, sandbox, auth, runs, MCP, familiar fit), PBT, bugfix root cause | **Sonnet 5.5 (1.3x)** | Strong reasoning on the parts judges read |
| One deep sandbox security audit; any task that failed twice on Sonnet | **Opus 5.5 (2.0x)** | Rare, high stakes |
| Routine Rust tasks, glue, deploy plan, steering | **Luna (0.6x)** | Good value; keep context under 272K |
| Quick Specs, config, hook/agent JSON, docs, changelog, small UI wiring | **Haiku 5.5 (0.06x)** or **Qwen3 Coder Next (0.05x)** | Nearly free; keep context under 100K for Haiku |
| Unsure | Auto (1.0x) | Default router |

**Principle:** spend Kiro credits where judges see Kiro: specs, PBT, hooks, MCP/power, agents and steering. Boilerplate, CSS/animation polish, landing and break-room UI go to other tools (Part F). Kiro still writes *their* specs cheaply, so `/.kiro/specs` stays complete.

| # | Activity (in Kiro) | Model | Credits [E] |
|---|---|---|---|
| 1 | Steering: Generate Steering Docs + D.1 + 1 revision | Luna | 10 |
| 2 | Kiro creates hooks, agents, mcp.json and the power skeleton (D.2) | Haiku 5.5 | 3 |
| 3 | Analyze Requirements ×3 (ghost-engine, judge-sandbox, auth) | Sonnet 5.5 | 35 |
| 4 | Generate 6 hard specs (2, 4, 5, 6, 11, 12) with gates and revisions | Sonnet 5.5 | 140 |
| 5 | Generate 4 medium specs (1, 7, 8, 10); spec 7 is now Requirements-First [D 12 Nov] | Luna | 40 |
| 6 | Generate 4 light Quick Specs (3, 9, 13, 14) | Haiku 5.5 | 4 |
| 7 | **Core implementation tasks** (~60): hard ones (~25) on Sonnet 5.5, the rest on Luna | Sonnet 5.5 / Luna | 430 |
| 8 | **PBT:** property generation, running, fixing counterexamples | Sonnet 5.5 | 180 |
| 9 | **Bugfix specs** ×2–3 | Sonnet 5.5 | 80 |
| 10 | **Counterspell MCP + Power** | Sonnet 5.5 / Luna | 70 |
| 11 | Subagent reviews: rust-reviewer ×6 (Luna), sandbox-auditor ×2 (Sonnet 5.5) + 1 (Opus 5.5), 1 parallel run | mixed | 90 |
| 12 | Escalations to Opus 5.5 on stuck tasks (cap: 5) | Opus 5.5 | 80 |
| 13 | Debug/glue chat (Plan-mode deploy, Compose, Caddy, CI) | Luna | 180 |
| 14 | Interactive CLI: `/goal` ×2 + terminal reviews | Luna | 40 |
| 15 | Workflows recipe experiment ×1 | Luna | 20 |
| 16 | Kiro on UI: SSE client, ghost card, shrink animation and Bug Autopsy wiring, familiar state wiring only [D 12 Nov] | Haiku 5.5 / Qwen3 Coder Next | 20 |
| 17 | docs-scribe → `docs/KIRO_USAGE.md`, changelog summaries | Haiku 5.5 | 5 |
| 18 | Agent-type hooks | none (all hooks are free shell commands) | 0 |
| — | **Planned total** | | **1,427** |
| — | **Reserve** (overruns, freeze-day fixes, optional Crew) | | **~273** |
| — | **Total** | | **1,700** |

**Burn-down checkpoints (check the Kiro usage panel and screenshot it):**
- Wed 18 Nov: **≤ 600 used** (expect well under, because Sat 14 – Mon 16 is visuals by other tools [D 12 Nov]).
- Wed 25 Nov: **≤ 1,050**.
- Sun 29 Nov: **≤ 1,300**.
- Wed 2 Dec: **≤ 1,450**.

If you're over a checkpoint:
- move rows 7 and 13 from Sonnet to Luna, and from Luna to Haiku 5.5 / Qwen3;
- drop row 12;
- move more UI work to the other tools.

Never cut rows 3, 8, 9 or 10, because they're the judging evidence.

**Credit rules:**
1. Use a new session per spec task, so context stays small. Haiku 5.5 costs 5× more above 100K tokens, and Luna doubles above 272K.
2. Restore a **checkpoint** rather than re-prompting after a bad turn.
3. Use Opus only for the audit and real escalations. Don't use it to "make it nicer".
4. Keep every hook a shell command (0 credits).
5. Add `.kiroignore` for `target/`, `node_modules/`, `data/`, `*.csv` and `docs/kiro-log/chats/`.
6. Don't run Run-all-tasks on big UI specs. UI goes to the other tools.
7. Lower the reasoning effort for routine tasks where the picker offers it; reasoning effort affects credits.

---

# PART C: Day-by-day (SAST)
**Build order [D 12 Nov]: landing and visuals first, then the core.** Brighton chose this over the critique's "core first". The risk is a late core, so there's a **hard checkpoint on Wed 25 Nov 22:00 SAST**: on the VPS, a real wrong Python submission goes through the Postgres queue and the sandbox and shows a real minimized counterexample on the ghost card over SSE. **If it isn't working by then, all visual work pauses until it is.** Before the core works, visual prototypes use clearly labelled design fixtures, never fake results presented as real runs. Feature freeze stays **Wed 2 Dec**; submit by **Fri 4 Dec 20:00 SAST**.

| Day | Kiro work | Kiro features shown |
|---|---|---|
| **Thu 12 Nov** | Create the repo (MIT). Copy `docs/` brief + AGENTS.md. **Generate Steering Docs** → prompt D.1 → review. Ask Kiro to create the hooks from table A.4 (D.2) and test guard-secrets. **Spec 1 `repo-skeleton`** (D.3), Design-First; Run all tasks. **Plan mode** (D.18) for the OVH VPS, Compose and the rootless sandbox daemon. | steering modes, file refs, AGENTS.md, hooks, Design-First, Run all tasks, Plan mode |
| Fri 13 Nov | Finish the skeleton, deploy hello-world. Add GitHub MCP + Context7 power. **Spec 13 `landing-steel-night`** (D.15, Quick Spec on Haiku 5.5). The other tool starts the landing page from it. First chat export. | MCP, Powers, Quick Spec, cross-tool, export |
| Sat 14 Nov | **Visuals.** **Spec 7 `ide-workspace`** (D.9, Requirements-First on Luna). The other tool builds the `[UI]` shell (layout, Monaco "Crypt" theme, Steel Night tokens) on labelled fixtures. Landing continues. | Requirements-First, cross-tool |
| Sun 15 Nov | **Visuals.** The other tool builds the familiar's SVG art and 5 mood animations, and prototypes the ghost card, shrink animation and Bug Autopsy on labelled fixtures. In Kiro: **Spec 2 `ghost-engine`** (D.4) requirements → **Analyze Requirements** → design with **Correctness/PBT** (spec only). Sunday usage check. | Analyze Req, PBT |
| Mon 16 Nov | **Visuals done:** the landing page is live on the VPS and the IDE shell renders. Start ghost-engine tasks, supervised. | |
| Tue 17 Nov | Ghost-engine tasks. **Spec 3 `grimoire-packs`** (Quick Spec) + the first 3 test packs. Turn on hooks #3 and #10. | Quick Spec, test-on-save |
| Wed 18 Nov | **Spec 4 `judge-sandbox`** (D.6): Design-First Low-Level + PBT limits; Python only; rootless sandbox daemon; worker-owned verdicts. **sandbox-auditor** subagent run #1. Usage check (≤ 600). | custom agent, subagent |
| Thu 19 Nov | Sandbox tasks and the hostile-program tests. Hook #7 (dangerous shell) proves itself. First **Bugfix spec** if something breaks (likely the verdict mapping). | Bugfix spec |
| Fri 20 Nov | **Spec 5 `run-lifecycle`** (D.7): Postgres job queue, `run_events`, SSE replay. Postgres MCP on the dev DB. | MCP (Postgres) |
| Sat 21 Nov | Run-lifecycle tasks (claim/lease, reaper, terminal write, replay). | PBT |
| Sun 22 Nov | **Soft check:** a wrong Python solution gives a minimized counterexample through the sandbox, locally. rust-reviewer + sandbox-auditor **in parallel**. | parallel subagents |
| Mon 23 Nov | Deploy the core to the VPS. Kiro wires the IDE to the real API: SSE client and ghost card (spec 7 non-UI tasks). Playwright MCP screenshots. | cross-tool |
| Tue 24 Nov | **Spec 6 `auth`** (D.8), Requirements-First + Analyze: three sign-ins, no automatic linking, judge demo account. Hooks #5, #6 and #16 matter here. | security hooks |
| **Wed 25 Nov HARD CHECKPOINT (22:00 SAST)** | Real fail → ghost on the VPS (see above). **If it fails, pause all visual work** until it passes. sandbox-auditor run #2. Usage check (≤ 1,050). | custom agent, evidence |
| Thu 26 Nov | **Spec 8 `language-drivers`** (D.10): JavaScript. **Spec 9 `hints`** (Quick). The **problem-author** agent + counterspell MCP bring the catalogue to **10 polished problems**. CLI `/goal` "Python and JS pass the hello problem". | CLI, /goal |
| Fri 27 Nov | **Spec 11 `counterspell-mcp-power`** (D.13): sandboxed execution, package the **Counterspell Power**, import it from GitHub, and record the **Kiro dogfooding demo**. Hook #4 goes live. **Spec 12 `efficiency-familiar`** (D.14): measured mood + PBT first, then the AI speech bubble. | Powers, MCP, hooks |
| Sat 28 Nov | Familiar wiring onto the existing art. Shrink animation and Bug Autopsy wired to real shrink traces. **Spec 10 `dashboard-fsrs`** (D.12, basic dashboard). Second **Bugfix spec**. Workflow recipe experiment. | Workflows |
| Sun 29 Nov | Dashboard finished; polish; core freeze-safe check. Usage check (≤ 1,300). | |
| Mon 30 Nov | **Stretch only if the core is freeze-safe:** Java and C++ (`[stretch]` tasks in spec 8), FSRS (`[stretch]` tasks in spec 10), live typing guess, **Spec 14 `break-room`** (Quick Spec on Haiku 5.5; the other tool implements it). Cut freely. | cheap-model routing |
| Tue 1 Dec | Plan mode: hardening + deploy. Security power scan (if it works without a paid account). | Powers (security) |
| **Wed 2 Dec FREEZE** | `/release-checklist`. sandbox-auditor run #3 on Opus 5.5. Load test (20 concurrent submits; decide if `JUDGE_CONCURRENCY` can rise above 2). Usage check (≤ 1,450). docs-scribe drafts `docs/KIRO_USAGE.md` from the logs. Final `/.kiro` tidy. | manual steering, custom agent |
| (Thu 3 – Fri 4 Dec) | Video + Devpost (outside Kiro scope); **submit by Fri 4 Dec 20:00 SAST**. Optional Crew stretch only if there's time. | Crew |

---

# PART D: Paste-ready prompts
> These are prompts, not specs. Kiro writes every requirements / design / tasks file. Each prompt references `docs/` so Kiro reads the brief itself. For any feature spec, also add `#[[file:docs/requirements.md]]` and name the story IDs (e.g. GHO-1, FAM-1) so Kiro reuses our EARS criteria.

## D.1 Steering (after clicking "Generate Steering Docs")
```
I've added our product brief to docs/: #[[file:docs/product-brief.md]] #[[file:docs/requirements.md]] #[[file:docs/system-design.md]] #[[file:docs/palette.md]] #[[file:docs/licence-rules.md]]

1) Rewrite the generated .kiro/steering/product.md, tech.md and structure.md so they match these docs. Keep each under ~80 lines and reference the docs with #[[file:...]] instead of copying long sections. In tech.md, add a "Decided – do not change without asking Brighton" list (Rust/Axum single binary + trusted judge worker that owns verdicts, Postgres incl. the job queue (FOR UPDATE SKIP LOCKED + leases) and run_events, Valkey for cache and rate limits only, Docker via bollard on a separate rootless sandbox-only daemon, the canonical Cargo layout counterspell-{server,judge,engine,grimoire,mcp,common}, Python + JavaScript at launch with Java/C++ stretch, SvelteKit static, Bits UI, Tailwind 4, GSAP only for landing + familiar, Monaco, MIT licence, OpenAI-compatible hints with gpt-6-luna then Groq gpt-oss-20b).
2) Create these steering files with correct front-matter inclusion modes:
   - rust-teaching.md (always): I am new to Rust. After any Rust change, add a short "Rust lesson" (3-5 lines, plain English). No unwrap()/expect() outside tests. Prefer simple code over clever generics.
   - security.md (always): never read or write .env or keys; ownership checks on every query; OAuth state+PKCE; no automatic account linking by email; rate limits; follow docs/system-design.md §7 for the sandbox; untrusted code never runs outside the sandbox, including from counterspell-mcp.
   - sandbox.md (fileMatch: crates/judge/**, languages/**, deploy/**): summarise §5–7 of the system design: the trusted worker owns tests, expected outputs, comparator, per-test deadlines (killed from outside) and the final verdict; user code only prints answers; sandboxes run only on the rootless sandbox daemon.
   - ghost-engine.md (fileMatch: crates/engine/**): determinism, seeded RNG, shrink invariants (each accepted step strictly reduces the size metric), proptest conventions; call results "minimized counterexamples", never "smallest possible".
   - frontend.md (fileMatch: web/**): Svelte 5 runes, Bits UI, Tailwind tokens from docs/palette.md, dark only, reduced-motion, GSAP only in landing/familiar, calm IDE.
   - grimoire-authoring.md (fileMatch: packs/**): problem pack format, original problems only.
   - api-conventions.md (auto, with name + description): JSON errors, cookie sessions, SSE event names from §9.
   - kiro-usage-log.md (manual): when invoked, append what we just did with Kiro (feature, surface, hooks, agents, MCP, what worked or not) to docs/KIRO_USAGE.md.
   - release-checklist.md (manual): the freeze-day checklist from §14 + §7.
3) Create a 6-line AGENTS.md at the repo root pointing to docs/ and .kiro/steering/, and crates/judge/AGENTS.md saying this crate is the security boundary.
Ask me questions before writing if anything in the docs conflicts.
```

## D.2 Hooks (Kiro writes the JSON + scripts)
```
Create agent hooks in .kiro/hooks/ (schema "version": "v1") and the scripts they call in scripts/kiro-hooks/, following this plan: #[[file:docs/kiro-playbook.md]] section A.4 (hooks table). Rules: use "command" actions everywhere (no credits); task-done-changelog is a command that appends the spec/task ID, commit hash and verification result to docs/kiro-log/tasks.md. Exit codes: any nonzero exit blocks PreToolUse, UserPromptSubmit (IDE) and PreTaskExec hooks, so use exit 2 only where we mean to block, and print to stdout and exit 0 where we only want to inform (pre-task-green and ghost-hunt-on-test-failure never block). Scripts are bash with `set -euo pipefail` (or POSIX sh that captures the real command's status before any pipe), read the event JSON from STDIN, and use jq where possible. The SPDX hook must use `<!-- SPDX-License-Identifier: MIT -->` in .svelte files and `//` only in .rs/.ts. Then show me how to test guard-secrets (writing .env must be blocked, .env.example allowed), dangerous-shell (docker run --privileged must be blocked), and that a failing `cargo test` piped through `tail` still reports failure.
```
*(Copy `kiro-playbook-2026-11-11.md` into the repo as `docs/kiro-playbook.md` so this reference works.)*

## D.3 Spec 1: `repo-skeleton` (Feature, Design-First, Low-Level)
```
Create a feature spec named repo-skeleton using the Design-First workflow (low-level design), because the architecture is already decided in #[[file:docs/system-design.md]] (§2 Architecture, §3 Components, §10 Deployment, §14 first milestone).
Scope: Cargo workspace with exactly the layout in §2 (crates/server, judge, engine, grimoire, mcp, common, published as counterspell-server, counterspell-judge, counterspell-engine, counterspell-grimoire, counterspell-mcp, counterspell-common; no other crate names), Axum server serving the SvelteKit adapter-static build and /api/health, SvelteKit app in web/ with Tailwind 4 tokens from docs/palette.md, docker-compose with postgres + valkey (cache and rate limits only) + caddy, a deploy/ note for the separate rootless sandbox daemon, sqlx migrations folder, GitHub Actions CI (fmt, clippy, test, svelte-check, cargo deny licenses), MIT LICENSE, THIRD-PARTY-NOTICES.md.
Done means: `docker compose up` serves the Svelte page from the Rust binary at localhost and CI is green. Keep tasks small (≤ 30 min each) and mark which tasks can run in parallel.
```

## D.4 Spec 2: `ghost-engine` (Feature, Requirements-First, PBT on)
*Rewritten 12 Nov from critique §5.5, with Brighton's decisions.*
```
Create a Requirements-First feature spec named ghost-engine. Context: #[[file:docs/system-design.md]] §8 Ghost engine and §5 ghost hunt protocol, #[[file:docs/requirements.md]] GHO-1, GHO-5, GHO-6, and #[[file:docs/product-brief.md]].
Goal: discover, reproduce and minimize counterexamples for incorrect solutions. Build a small Rust library (crates/engine, package counterspell-engine) with three clearly separated parts: input generation, failure classification, and shrinking. The pure shrinking algorithm must be separate from code execution: the engine talks to programs only through an Oracle trait that the judge implements with the sandbox. Nothing in this crate may execute code itself.
Initial supported inputs: bounded integers, strings, integer arrays, 2-D integer arrays and tuples of these (the §8 schema). No graphs, trees or arbitrary executable custom checkers in this spec.
Correctness requirements:
1. Every generated candidate satisfies the problem's input constraints.
2. A failure is independently verified against the trusted Python reference by the judge's comparator.
3. During shrinking, a candidate is kept only if it still reproduces the same failure predicate (verdict kind + exception class).
4. Define an explicit deterministic size metric (e.g. element count, then sum of absolute values, then lexicographic) and accept only steps that strictly reduce it.
5. Edge-biased deterministic generation; record seed, generator version, runtime image digest and problem version with every result.
6. Support wrong-answer and runtime-error counterexamples first; handle timeouts separately.
7. Stop when the candidate, evaluation or wall-time budget is reached (total ghost hunt ≤ 8 s).
8. Return a "minimized counterexample". Never claim global minimality.
9. Return a clear "not found / budget exhausted" status when no counterexample is found.
10. Record every accepted shrink step in order (a capped shrink trace) for the UI's shrink animation, and return the data the Bug Autopsy card needs (input, expected, actual, kind, concept tags, matching known-ghost explanation if any).
Property-based tests (enable Correctness/PBT in design, using proptest) must verify: generated candidates satisfy their validators; every returned ghost reproduces its claimed failure; every accepted shrink step reduces the size metric; equal seeds and equal deterministic oracle behaviour give equal results; the search terminates within its budgets.
Create three test problem packs, each with a correct Python reference and at least two deliberately incorrect implementations (one of them is reused later for the Kiro dogfooding demo).
Out of scope: AI hints, UI animations, complexity profiling, more language drivers.
Definition of done: three incorrect programs produce valid, reproducible minimized counterexamples within budget, and every result passes independent re-verification.
I'm new to Rust: keep the API small and explain the design trade-offs in plain English in design.md.
```
Then click **Analyze Requirements** before approving.

## D.5 Spec 3: `grimoire-packs` (Quick Spec)
```
Quick spec: grimoire-packs. Implement the problem pack format from #[[file:docs/system-design.md]] (§4 data model problem fields, §8) as files under packs/<slug>/ (problem.md, meta.toml, reference solutions, generator/shrinker hooks into crates/engine) plus a `counterspell-grimoire validate <pack-dir>` command (crate crates/grimoire) and the first original starter problems toward the 10 polished launch problems (no HackerRank/LeetCode text). Every pack has a Python reference (authoritative for expected outputs), authored hints, and optional known-ghost seeds with an authored Bug Autopsy explanation. Follow .kiro/steering/grimoire-authoring.md.
```

## D.6 Spec 4: `judge-sandbox` (Feature, Design-First Low-Level, PBT on)
*Rewritten 12 Nov from critique §5.4, with Brighton's decisions.*
```
Create a Design-First (low-level) feature spec named judge-sandbox. Read #[[file:docs/system-design.md]] §2, §5 Run lifecycle, §6 Languages as config, §7 Sandbox security and §7.1, and #[[file:docs/requirements.md]] (Sandbox criteria, VAL-1–VAL-5). Treat the security requirements below as authoritative if the docs disagree, and ask me before choosing anything that weakens isolation.
Objective: a Python-only execution service in crates/judge (package counterspell-judge) that safely runs hostile user code and produces verified results. JavaScript, Java, C++ and bench mode are outside this spec; languages come from languages/languages.toml so others are config later.
Trust boundaries:
- Axum (crates/server) never accesses any Docker daemon.
- Sandbox containers run on a separate rootless Docker daemon (unprivileged `sandbox` user, own socket) used only for sandboxes. The app, Postgres, Valkey, Caddy and secrets are not on it. The judge mounts only that socket and has no access to the main daemon socket.
- The judge worker is trusted. Submitted code, language drivers and everything they print are untrusted.
- The worker owns test cases, expected outputs, the comparator, per-test time limits and the final verdict. The driver feeds one input at a time and prints only the raw answer. A submitted program must never be able to declare its own verdict; text that looks like a verdict is just an answer to compare.
- No user-controlled code runs on the host, including through counterspell-mcp.
Implementation requirements:
1. Fixed, digest-pinned images and fixed command arrays. Never interpolate submitted source or untrusted input into shell commands.
2. cgroups v2 CPU, memory, swap and PID limits; read-only rootfs; bounded tmpfs scratch; non-root UID; network none; cap-drop ALL; no-new-privileges; default seccomp. `--cpus` is a quota, not pinning. Verify that rootless Docker actually applies these limits (cgroup v2 delegation) on the target host.
3. Per-test deadlines and the total wall-time cap are enforced by the worker from outside (kill), not by the driver.
4. Bounded streaming of stdout/stderr (1 MB / 64 KB) with an explicit output-limit verdict.
5. Separate compile, execute, compare and cleanup responsibilities.
6. Sandbox infrastructure failure returns an explicit internal error, never a user's wrong answer.
7. Worker crash recovery and stale-container cleanup (reaper within 60 s).
8. gVisor stays selectable by configuration, but don't claim it works until tested on the target host.
Required tests (hostile-program corpus, as correctness properties where possible): correct and incorrect solutions; infinite loop; memory exhaustion; process exhaustion (fork bomb); output flood; network access attempt; filesystem write outside scratch; user-printed fake verdict JSON; worker termination during execution; timeout and cleanup; multiple concurrent submissions (JUDGE_CONCURRENCY=2 default). Properties: verdict is never Accepted for the hostile corpus, the container is always removed, wall time ≤ limit + 1 s, no network for any destination.
First produce the threat model and design, with assumptions and trade-offs. Don't implement until I've reviewed the trusted/untrusted boundary. Run Analyze Requirements first.
Definition of done: a correct Python solution passes, a wrong one fails, every hostile test is contained within its documented limits, no user-controlled output can forge Accepted, and no orphaned container remains after recovery. Report the behaviour you actually tested, not assumed guarantees.
Also cover pre-sandbox submission validation (enforced by the API server before a run is queued; see #[[file:docs/requirements.md]] VAL-1–VAL-5 and #[[file:docs/system-design.md]] §7.1): client checks are UX only and the server repeats every check; size limit in bytes (configurable per language, default 64 KB), not lines; tree-sitter parse for the selected language, reject unparseable code before queuing; plain text only (valid UTF-8, no NUL/control characters except tab/newline/CR, single source file, no uploads); per-user and per-IP rate limits in Valkey plus a max queued jobs per user; stdout/stderr and test output rendered as plain text, never HTML, truncated to a display cap. Do not use keyword blocklists. The sandbox remains the real security boundary.
```

## D.7 Spec 5: `run-lifecycle` (Feature, Design-First High-Level)
*Rewritten 12 Nov from critique §5.6, with Brighton's decisions. Renamed from `runs-streams-sse`.*
```
Create a Design-First (high-level) feature spec named run-lifecycle. Context: #[[file:docs/system-design.md]] §2, §4 (jobs, run_events), §5 Run lifecycle, §9, and #[[file:docs/requirements.md]] IDE-3 and VAL-4.
Use PostgreSQL as the authoritative store for submissions, the job queue and run events. Valkey is used only for rate limits and caches; do not use Valkey Streams or pub/sub as a queue or event bus.
Required behaviour:
1. Creating a submission and its claimable job is atomic (one transaction); jobs.submission_id is unique and POST /api/runs honours an Idempotency-Key.
2. A worker claims a job exclusively with SELECT … FOR UPDATE SKIP LOCKED and a renewable lease.
3. A worker crash never leaves a submission permanently running: a reaper requeues expired leases with attempts + 1, and after max_attempts the job is dead and the submission an internal error.
4. Retries are idempotent: the terminal write (final verdict, test results, done event, job succeeded) is one transaction guarded by the current lease owner, so duplicate terminal results are impossible.
5. Every public event is persisted in run_events with a monotonically increasing per-run seq. A retried attempt appends a `restarted` event first.
6. The SSE endpoint authorizes ownership before exposing any event.
7. The SSE id is seq; on reconnect the server replays events with seq > Last-Event-ID from Postgres, then waits on LISTEN/NOTIFY (a wake-up hint only) with a short poll fallback.
8. The client deduplicates by seq.
9. Hidden test inputs and expected answers are never in public event payloads.
10. A finished run has exactly one authoritative final result and terminal state, fetchable after page reload via GET /api/runs/{id}.
Define the full run state machine: queued, claimed, compiling, running, ghost_hunting, completed (with verdict), failed, expired, internal_error.
Include correctness properties and integration tests for concurrent workers, duplicate claims, lost SSE connections, restart recovery, cancellation and event replay. Per-user rate limits and the queued-jobs cap come from §7.1.
Definition of done: twenty concurrently submitted lightweight jobs are all processed with no lost jobs, no duplicate terminal results and no unauthorized event access. Report measured enqueue and completion latencies separately.
```

## D.8 Spec 6: `auth` (Feature, Requirements-First, Analyze + PBT)
```
Create a feature spec named auth (Requirements-First). Context: #[[file:docs/system-design.md]] §1, §4 users/sessions tables, §9 auth endpoints; email via Resend. GitHub + Google OAuth (state + PKCE) and email magic link (single-use, 15-min expiry), cookie sessions (HttpOnly, Secure, SameSite=Lax), logout everywhere. Keep all three sign-ins, but NO automatic account linking by matching email: linking happens only when a signed-in user starts it, via OAuth with a link-intent state bound to the session or a verification email (#[[file:docs/requirements.md]] ACC-1, ACC-3). Add the judge demo account path (ACC-6): seeded demo user, sign-in link on the login page, can't link providers, change credentials or delete itself, daily reset. Write EARS criteria that yield properties (token single-use, expiry, a session never reads another user's data, an OAuth email match never links or signs in to another account). Run Analyze Requirements. Never touch .env; read secrets from env vars only.
```

## D.9 Spec 7: `ide-workspace` (Feature, Requirements-First)
*Rewritten 12 Nov from critique §5.7 ("ghost workspace"), with Brighton's decisions. This is the main Costume Contest showcase.*
```
Create a Requirements-First feature spec named ide-workspace. This is the central Counterspell experience and the main Costume Contest showcase. UI will be implemented partly outside Kiro, so keep tasks granular, label UI-only tasks "[UI]", and mark independent UI tasks so they can run in parallel.
Context: #[[file:docs/product-brief.md]] core loop, #[[file:docs/requirements.md]] IDE-1–IDE-6, GHO-1–GHO-6, FAM-1, FAM-3–FAM-6, VAL-5, #[[file:docs/system-design.md]] §5, §8, §8b, §9 and §16, and #[[file:docs/palette.md]] (Monaco "Crypt" theme). Use SvelteKit, Svelte 5, Monaco, Bits UI, Tailwind 4 and GSAP. Follow Steel Night. No React, Next.js or Pyodide.
User journey:
1. Open a problem with a clear statement, constraints and starter function (Python or JavaScript at launch; Java/C++ appear only if the stretch drivers ship).
2. Write and edit without losing drafts (per problem and language).
3. Run sample tests and inspect input / expected / got.
4. Submit and receive real execution status updates over SSE (replay with Last-Event-ID, dedupe by seq).
5. On a wrong answer, animate the transition into Ghost Hunting.
6. Show the visible shrink animation: replay the recorded shrink steps from the first failing input down to the minimized counterexample (only real recorded steps; static list under reduced motion).
7. Show the Bug Autopsy card: failing input, expected vs actual with the differences highlighted, and the conceptual mistake (authored explanation, or an AI explanation labelled as AI).
8. Offer escalating authored hints and a button to rerun the ghost as a custom test.
9. Let the learner correct the code and resubmit.
10. Show a successful resolution, the familiar's measured mood and its labelled AI speech bubble, and update progress.
Design requirements: a cinematic but functional ghost reveal; never animate or display fabricated execution results (design fixtures are allowed only in clearly labelled prototype routes); calm, fast, keyboard-accessible editor interactions; reduced-motion equivalents; status labels that don't rely on colour alone; output rendered as plain text only; the familiar's mood comes only from measured run events (no live typing guess in this spec); lazy-load Monaco and heavy animation assets; usable on a mid-range laptop and a tablet.
Deliverables: interaction flow, responsive layout, component boundaries, animation state machine, loading/error/empty states, keyboard behaviour and accessibility acceptance tests.
Build and test against the actual Axum API, not mocked success responses (the early [UI] shell may use labelled fixtures until the API exists).
Definition of done: a judge can submit a wrong solution, watch the real shrink, read a real minimized counterexample in the Bug Autopsy, fix the code and reach Accepted without a page reload or manual intervention.
```

## D.10 Spec 8: `language-drivers` (Feature, Design-First)
```
Create a feature spec named language-drivers (Design-First). Context: #[[file:docs/system-design.md]] §6 Languages as config and §8b bench mode. Add JavaScript (node) to languages.toml with run commands and a stdin/stdout driver that matches the Python driver, plus bench mode for the Efficiency Familiar. Drivers print only raw answers; the worker compares and times. Java (eclipse-temurin, not openjdk; -Xss16m to start, test 8–16m on deep-recursion problems and keep the smallest that passes) and C++ (gcc) are tasks labelled [stretch], done only after the core is freeze-safe. The Python reference is authoritative for expected outputs in every language; there are no per-language references. Properties: for every problem, each driver round-trips any generated input and output exactly, and a per-language known-correct test fixture matches the Python reference's outputs on generated inputs.
```

## D.11 Spec 9: `hints` (Quick Spec)
```
Quick spec: hints. Context: #[[file:docs/system-design.md]] (hint ladder, AI hints) and #[[file:docs/licence-rules.md]]. Authored hint ladder per problem, plus an optional AI hint grounded in the ghost input via one OpenAI-compatible client: primary gpt-6-luna, fallback Groq openai/gpt-oss-20b on error/timeout; per-user daily cap; never send the full solution; label AI output as an AI hint. The same client serves the familiar's speech bubble (spec 12). Keys from env vars only.
```

## D.12 Spec 10: `dashboard-fsrs` (Feature, Requirements-First, PBT)
```
Create a feature spec named dashboard-fsrs (Requirements-First). Context: #[[file:docs/system-design.md]] §1, §4, §9 and #[[file:docs/product-brief.md]] step 6. Dashboard (Must): solved count, streak, weak concepts, suggested challenges. Spaced repetition (next reviews) is a stretch goal: label those tasks [stretch] and don't start them before the core is freeze-safe. Use the `fsrs` crate (BSD-3) — NOT rs-fsrs (see docs/licence-rules.md). Properties: next interval non-negative; grows after "good"; dashboard only shows the signed-in user's data.
```

## D.13 Spec 11: `counterspell-mcp-power` (Feature, Design-First)
```
Create a feature spec named counterspell-mcp-power (Design-First). Context: #[[file:docs/system-design.md]] §8 and the MCP/Powers plan in #[[file:docs/kiro-playbook.md]] A.5–A.6.
1) crates/mcp: an MCP server using the official Rust SDK (rmcp) exposing validate_pack, find_ghost_input, shrink_input, run_reference, explain_verdict, fit_complexity, reusing crates/engine (no duplicated logic). stdio transport. Every tool that executes code (run_reference, find_ghost_input, shrink_input) goes through the judge crate's sandbox runner and trusted comparator on a sandbox-only rootless Docker daemon, with the same images and limits as production; it must never spawn python/node directly on the developer's machine. Tool descriptions say so.
2) kiro-power/: a Kiro Power in the Agent Plugins format (plugin.json with keywords, mcp.json pointing at the server, skills/author-problem/SKILL.md and skills/hunt-ghost/SKILL.md).
3) Register the server in .kiro/settings/mcp.json with autoApprove only for read-only tools.
4) Kiro dogfooding demo (#[[file:docs/requirements.md]] MCP-3): a demo pack with a deliberately buggy reference or implementation; a scripted session where Kiro calls find_ghost_input, gets a minimized counterexample through the sandbox, fixes the bug, and re-verifies with validate_pack + find_ghost_input. Save the chat export and screenshots for the video and docs/KIRO_USAGE.md.
```

## D.14 Spec 12: `efficiency-familiar` (Feature, Requirements-First, PBT)
```
Create a feature spec named efficiency-familiar (Requirements-First). Context: #[[file:docs/system-design.md]] §8b The Efficiency Familiar (moods, measured layer, speech bubble, limits, Postgres fields, API/run events), #[[file:docs/requirements.md]] FAM-1–FAM-6, #[[file:docs/palette.md]], and the jack-o'-lantern art in web/static/familiar/ (the SVG art and mood animations may already exist from the visuals phase; reuse them).
Rules: the mood is set ONLY by measured facts — correctness plus observed scaling vs reference_complexity, with confidence. Word results as observations ("looks quadratic, medium confidence"); the top mood is "Matches reference complexity", never "optimal".
Build order: (1) the measured layer (bench mode timings measured by the worker → complexity fit vs reference → mood); (2) wire the mood to the existing SVG + GSAP mascot (5 moods, laugh burst, crimson crack-and-burst; reduced motion, mute and hide); (3) the speech bubble: POST /api/runs/{id}/familiar-hint, called once per finished Run/Submit (never on keystrokes), using the hints client (gpt-6-luna, then Groq gpt-oss-20b), grounded only on the code, measured results and reference complexity, labelled "AI hint", unable to change verdict or mood, with a templated fallback; (4) [stretch] the live-typing heuristic in a web worker, shown only as a labelled guess that never changes the mood.
Properties for the fit: on synthetic n / n log n / n² timings with noise it returns the right class or "unsure", never a wrong class with high confidence.
```

## D.15 Spec 13: `landing-steel-night` (Quick Spec)
```
Quick spec: landing-steel-night. Context: #[[file:docs/system-design.md]] §16 Look and feel and #[[file:docs/palette.md]]. Cinematic landing page: moon/graveyard hero with separate SVG assets, GSAP ScrollTrigger reveals, the ghost-input idea explained in 3 beats, CTA to sign up, Geist + JetBrains Mono, no serif, dark only, reduced-motion fallback, Lighthouse performance ≥ 90 on mobile.
```

## D.16 Spec 14: `break-room` (Quick Spec, built last)
```
Quick spec: break-room (lowest priority; stretch, don't start before the core is freeze-safe; can be cut). Context: #[[file:docs/system-design.md]] break room notes and #[[file:docs/licence-rules.md]]. Daily chess puzzle from the CC0 Lichess puzzle CSV (imported once, no AGPL code), chess.js + cm-chessboard, a simple minimax bot (no Stockfish), and one small brain game. Keep it isolated under /break so cutting it is one route.
```

## D.17 Bugfix spec template
```
Create a bugfix spec named bugfix-<short-name>. Current behaviour: <what happens, with the failing input or log>. Expected behaviour: <what should happen>. Unchanged behaviour: <what must keep working>. Use #[[file:<relevant file>]] and #terminal. Include a property test that reproduces the bug before the fix.
```

## D.18 Plan mode: deploy
```
(Plan mode) Read #[[file:docs/system-design.md]] §10 Deployment and §11 Cost. Produce a step-by-step plan to deploy to an OVH VPS-2 (4 vCPU, 8 GB, 75 GB) with Docker Compose + Caddy (HTTPS) on the main daemon, a Postgres volume and off-box backups, Valkey for cache and rate limits only, a separate rootless Docker daemon for sandboxes only (unprivileged `sandbox` user, cgroup v2 delegation verified, images pre-pulled), the judge mounting only that rootless socket with no access to the main daemon, JUDGE_CONCURRENCY=2, and a GitHub Actions deploy. Don't change files; list commands I run myself.
```

---

# PART F: Using other coding agents alongside Kiro (rules check + work split)
## F.1 What the rules say (live pages opened 11 Nov 2026, 23:50 SAST)
The **Official Rules** (kiroween.devpost.com/rules) and the **FAQ** (kiroween.devpost.com/details/faqs) **don't mention other AI coding agents or tools at all**: no Codex, Cursor, Devin, Claude Code, Copilot or "other AI tools". No clause permits them and none forbids them. The relevant verbatim text:

- **Rules, Project Requirements:** "Entrants must build a working software application using Kiro (IDE, CLI, Web, and/or Kiro Crew) and submit it into one of the following categories (each a “Project”). **Insufficient use of Kiro will result in disqualification at the Judges’ sole discretion.**"
- **Rules, New Projects Only:** "Participants may use standard development resources, including frameworks, libraries, starter templates, but must disclose any other pre-existing code or work incorporated into the Project. The work described and submitted must have been built during the Submission Period."
- **Rules, Submission Requirements:** "Your repo must contain the /.kiro directory at the root of the project to show usage of Kiro such as specs, hooks, and steering. Do NOT add the /.kiro directory or sub-folders to your .gitignore, as this could disqualify your submission."
- **Rules, Intellectual Property:** "An Entrant may contract with a third party for technical assistance to create the Submission provided the Submission components are solely the Entrant’s work product and the result of the Entrant’s ideas and creativity, and the Entrant owns all rights to them."
- **Rules, judging (Technological Implementation):** "How thoroughly and skillfully does the project use Kiro? Was there a thoughtful strategy to AI development? Was a variety of Kiro tools used impactfully? Does the project show a deep understanding of Kiro features? Was there experimentation?"
- **FAQ:** "**Do I have to use Kiro?** Yes. Your project must be a working software application that uses Kiro (vibe coding, agent hooks, steering docs, MCP, or specs). Your repo must also contain the /.kiro directory at the root."
- **Not checked:** Discord #kiroween-hackathon and Devpost "Updates" posts (the Updates page had no mention). If in doubt, ask in Discord. That's Brighton's call; I contacted no one.

**Safe interpretation (my reading, not a rule):** other tools aren't prohibited, but:
1. Kiro must be **meaningfully** used. "Insufficient use" is a judge's-discretion disqualifier.
2. `/.kiro` must be present and rich (specs, hooks, steering, agents, settings).
3. **Disclose** other tools in the README and the Kiro write-up. It's honest, and the "disclose … other … work" spirit applies.
4. Keep the **core and novel parts visibly Kiro-built**, so the Technological Implementation score (20%) isn't diluted.

## F.2 Work split
| Area | Who | Why |
|---|---|---|
| All **specs** (including UI ones), steering, hooks, custom agents, MCP config, Power | **Kiro** | This is the `/.kiro` evidence. Other tools may implement from Kiro's specs, but they never write specs. |
| Ghost engine, judge sandbox, run lifecycle (Postgres queue + SSE), auth, language drivers, counterspell-mcp + the dogfooding demo, familiar complexity fit and speech bubble, FSRS scheduler (stretch) | **Kiro** (spec tasks) | Core, novel, PBT-backed; what judges should see Kiro do |
| Bugfix specs for real bugs in Kiro-built code | **Kiro** | A bugfix-spec story for the write-up |
| Security audits, code reviews | **Kiro** custom agents (subagents) | Custom-agent evidence |
| IDE layout polish, landing page visuals/GSAP choreography (built first [D 12 Nov]), familiar SVG art and mood animations, shrink-animation and Bug Autopsy visuals, break room UI, CSS tweaks, SVG asset prep | **Other agent** (e.g. Cursor / Codex / Claude Code), working **from Kiro's Quick Spec tasks** | Saves credits; low judging weight |
| Repetitive boilerplate (seed data for 15 problems, fixtures, README copy edits) | Other agent, or Kiro on Haiku 5.5 / Qwen3 (≈ free) | Prefer Kiro if it costs under 1 credit, because it adds evidence |
| Deploy scripts | Kiro Plan mode first, then either tool | Plan mode is evidence |

## F.3 Keeping the `/.kiro` evidence strong when other tools help
1. **Specs stay authoritative.** Another tool works from a Kiro task and ticks nothing itself. Afterwards, open the spec in Kiro and use **Sync Files / mark tasks done** so `tasks.md` reflects reality. Log `[UI by <tool>]` next to the task in `docs/kiro-log/tasks.md`.
2. **Hooks still guard the result.** Other tools' edits don't fire Kiro file hooks, because hooks fire on agent edits in Kiro only. So run the same `scripts/kiro-hooks/*.sh` in a git pre-commit hook and in CI. That way the licence, secrets and fmt checks cover everyone, and the scripts are shared evidence.
3. **Review in Kiro.** After another tool's change, run `rust-reviewer` or a UI review subagent in Kiro on the diff, so Kiro checks the outsider's work.
4. **Branch and commit hygiene.** Use commit trailers such as `Kiro-Spec: ghost-engine#3` or `Assisted-by: <tool>`. A short `git log` breakdown can go in the write-up: "X% of commits from Kiro spec tasks".
5. **Shared rules.** Point the other tool at the same `AGENTS.md` and `docs/` brief (most agents read `AGENTS.md`), so the code style and security rules match Kiro's steering.
6. **Disclose.** Add a README "How this was built" section: Kiro for specs, core, PBT, hooks, MCP and agents; `<tool>` for UI polish implemented from Kiro specs. Use the same wording in the Devpost Kiro write-up.
7. **Video.** Show only Kiro on screen in the "how we built it" segment.

---

# PART E: Unverified items / open questions
1. **Resolved:** about 1,700 grant credits, all models available. Kiro Web and headless CLI are dropped, because the docs tie them to a paid subscription.
2. **Hook blocking exit code (updated 12 Nov):** the plan now treats **any nonzero exit as blocking** for PreToolUse, IDE UserPromptSubmit and PreTaskExec, uses 2 to block on purpose and 0 + stdout to inform (A.4). Still [U]: how CLI prompt-submit hooks handle nonzero exits. Test on day 1.
3. **The write-tool STDIN payload field names:** the scripts grep the whole JSON.
4. **Hook triggers in the CLI:** the trigger table and PascalCase IDs are verified on ide/whats-new-v1/hooks. CLI V3 supports the file and task triggers. Confirm in the IDE hook editor on day 1.
5. **Correctness/PBT:** listed as IDE-only. Specs created on CLI/Web may not get properties.
6. **Postgres MCP Pro flag** for read-only (restricted) mode. Check its README when installing.
7. **Whether Powers can bundle steering and hooks** (the marketing page says yes; the create docs list skills + MCP).
8. **Security powers (Aikido/Snyk)** may need an account. Context7 power availability is confirmed in the registry listing.
9. **Crew:** the prize needs "a public URL to the code repository for the app on GitHub", and the app must be newly created (rules verified). [U] Whether Crew needs a subscription and what it costs in credits.
10. **Model IDs** for `model` in agent JSON (e.g. `claude-sonnet-4.6` appears in the docs example; exact IDs for Sonnet 5.5, Luna and Haiku 5.5 need checking in the model picker).
11. **Credit costs per spec/task** are estimates. Measure after spec 1 and adjust Part B.
12. **Local command sandbox:** whether it interferes with Docker or stdio MCP. Test before turning it on.
13. **Other coding agents:** the rules and FAQ are silent (Part F). The safe reading is mine, not an official ruling.
14. **Rootless Docker limits (added 12 Nov):** rootless Docker needs cgroup v2 with delegation for memory, CPU and PID limits to apply. Verify on the OVH VPS image on day 1, and verify whether gVisor works with it before enabling it.
15. **Java `-Xss` (added 12 Nov):** `-Xss16m` is a starting point; test 8–16m on deep-recursion problems and keep the smallest that passes (stretch language).
16. **Judge concurrency (added 12 Nov):** 2 by default on 8 GB until the freeze-day load test; raise only with measured headroom.
