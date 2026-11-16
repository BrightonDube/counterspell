# Tasks: pack-authoring-pipeline

Status: approved for implementation by Brighton on 15 Nov 2026 10:26 (`brighton-decisions`, and `docs/kiro-log/pack-pipeline-blockers.md`). Each task is sized for 30 minutes or less. Skip tasks that need a purchase, a paid service, or the private repo Brighton has not created yet.

Markers:
- `[P]` can run in parallel with other tasks in its wave once dependencies are done.
- `[S]` sequential, or has a required dependency.
- `[SUPERVISED]` touches approval, hidden data, credentials, hooks or the publish path. Brighton reviews the diff before the task is ticked.
- `[HUMAN]` a step only Brighton or the operator can do (creating the private repo, approving a pack, running sandbox stages on the host).
- `[BLOCKED on …]` waits for the named spec or task. Record the block in `docs/kiro-log/pack-pipeline-blockers.md` and continue with work that does not depend on it.
- `[GATE]` a checkpoint. Work after it does not start until it is recorded.

Conventions for every task:
- Production Rust has no `unwrap()`, `expect()` or `unsafe`. Tests may `expect` with a message.
- Red, green, refactor. Write the failing test first and see it fail for the expected reason.
- Expected values in tests are hand-derived literals. Canary strings for hidden data are fixed values such as `CANARY-HIDDEN-pa7`.
- Every Rust task ends with a Rust lesson of 3–5 plain-English lines (`.kiro/steering/rust-teaching.md`).
- Gate before finishing each Rust task: `cargo fmt --all -- --check`, `cargo clippy -p counterspell-grimoire --all-targets -- -D warnings`, `cargo test -p counterspell-grimoire`.
- No task writes outside the files it names. No task creates the private GitHub repository, moves an existing pack file, or sets `status = "published"`.
- Tests make no network call, no GitHub call and no model call (PIP-49).

## Prerequisites and status of other specs

| Need | Spec and task | State at spec time |
|---|---|---|
| Grimoire static validator, `ProblemRecord`, CLI | grimoire-packs T12, T13, T14 | done |
| Grimoire layout for the two-tree split | grimoire-packs amendment X1 (design §14) | **not yet specified** |
| New grimoire codes (HID, STM-03, HNT-03, ORG, GHC, PUB, APR, REV, SUM, SLUG, TAX, BRF, LAY-05) | grimoire-packs amendment X2 | **not yet specified** (HID-* are in `33-starter-pack-hardening-v2.md` only) |
| Judge `PackRunner` | judge-sandbox T20 to T22, grimoire-packs T20 | open or blocked |
| Judge host checks (VPS) | judge-sandbox T02, T37 | `[HUMAN]` on the VPS |
| Code-running MCP tools | counterspell-mcp-power T10, T16 | open |
| MCP approval policy | counterspell-mcp-power CONFIG-2, CONFIG-3 (T14) | open |
| Importer reads both trees and checks `approval.toml` | problem-catalogue C14, C17 (amendment X3) | open; the 14 Nov 23:06 amendment is not visible in the spec |
| Operator CLI, publish | problem-catalogue C19, C21 | open |
| Efficiency bench enablement | efficiency-familiar T40 | open |
| Hint ladder parse and validation | hints G1, G2 | open |
| Starter packs `count-pairs`, `diagonal-total`, `longest-run-length` | grimoire-packs T21 to T23 (done, public hidden parts) | rotation needed (PIP-35) |

---

## Wave 0: approval

- [x] **PA00 [GATE] [HUMAN] (15 min) Approve this spec and its open questions**
  - Brighton reviews `requirements.md` and `design.md`. He answers design open questions 1 to 16. Priority: 1 (paid services), 3 (private repo and access), 4 and 5 (who sets `published`, where `approval.toml` lives), 6 and 7 (originality list and third-party reading), 9 and 10 (reviewer access and batch approvals), 12 (retired values).
  - Depends on: none.
  - Verify: `docs/kiro-log/pack-pipeline-blockers.md` records each answer or change. No later wave starts before it.

---

## Wave 1: pure pipeline model in grimoire

These tasks add a `pipeline` module to `crates/grimoire/`. The module is pure: no I/O, no process, no network, no sqlx. Hidden values are passed in memory and never returned.

- [x] **PA01 [S] (30 min) Stage and status model with state file parsing**
  - Files: `crates/grimoire/src/pipeline/mod.rs`, `crates/grimoire/src/pipeline/state.rs`, `crates/grimoire/tests/pipeline_state.rs`.
  - Define `Stage` (S0 to S13) and `PackStatus` (`in_progress`, `static_complete`, `sandbox_checked`, `in_review`, `rework`, `stale`, `passed`, `escalated`, `awaiting_approval`, `approved`). Define the allowed transitions from design §7. Parse and write `.kiro/pipeline/state/<slug>.toml` with serde and `deny_unknown_fields`.
  - Requirements: PIP-6, PIP-45, P10.
  - Depends on: PA00.
  - Verify: unit tests for each transition in design §7 and for a refused transition (for example `approved` back to `in_review`). A state file with a key named `hidden_expected` fails to parse.
  - Rust lesson: an enum with `match` makes the compiler list every state you must handle. Explain why an illegal transition returns `Result::Err` instead of panicking.

- [x] **PA02 [P] (30 min) Review scoring and pass rule**
  - Files: `crates/grimoire/src/pipeline/review.rs`, `crates/grimoire/tests/pipeline_review.rs`.
  - Input: seven category scores (1 to 5), a finding list (`id`, `category`, `severity`, `pointer`, `message`) and the content hash. Output: `Pass` or `Fail(Vec<RuleName>)`. Implement every rule in PIP-23 at its boundary. The output never repeats a message that could contain a pointer to hidden data; it names the rule only.
  - Requirements: PIP-22, PIP-23, PIP-24.
  - Depends on: none (can run with PA01).
  - Verify: boundary tests: a category at 2 fails; mean 3.9 fails and mean 4.0 passes; correctness 4 fails; an `ORG-*` finding fails; a `blocker` fails; an empty answer to a finding is rejected.
  - Rust lesson: `Result` and `Option` carry the failure reason in the type, so a fail can't be read as a pass by accident. Explain the difference between `Fail(Vec<...>)` and a boolean.

- [x] **PA03 [P] (25 min) Loop cap and escalation**
  - Files: `crates/grimoire/src/pipeline/loop_cap.rs`, `crates/grimoire/tests/pipeline_loop_cap.rs`.
  - Round counter, cap of three, and the transitions `rework`, `escalated` and `stale` from design §7. A new content hash starts a new round; the same hash does not.
  - Requirements: PIP-25, PIP-26, P4.
  - Depends on: PA01.
  - Verify: rounds 1 and 2 give `rework` on fail; round 3 gives `escalated`; a fourth round is refused with an error; re-running the same hash does not increment the counter.
  - Rust lesson: a counter that only goes up, with the limit checked in one function, is easy to test. Explain why the check lives in one place.

- [x] **PA04 [P] [SUPERVISED] (30 min) Originality marker guard (ORG-01, ORG-02)**
  - Files: `crates/grimoire/src/pipeline/originality.rs`, `crates/grimoire/tests/pipeline_originality.rs`, `crates/grimoire/tests/fixtures/pipeline/copied-marker/`, `crates/grimoire/tests/fixtures/pipeline/no-originality-record/`.
  - The marker list is passed in as data from the private repo. It is never embedded in code. Tests use synthetic markers only. Scan the public and private text, return codes and file pointers, never the matched text. Check that `originality.md` has the five sections named in S1 and at least one checklist answer per section.
  - Requirements: PIP-3, PIP-8, PIP-46, P6.
  - Depends on: PA01.
  - Verify: the two fixtures fail with exactly `ORG-01` and `ORG-02`. A canary marker in a test asserts that the matched text is absent from the report.
  - Rust lesson: `&str` borrows text without copying it, which helps when a function must scan secrets but never return them. Explain what the returned report owns.

- [x] **PA05 [P] [SUPERVISED] (30 min) Hint leak check (HNT-03)**
  - Files: `crates/grimoire/src/pipeline/hint_leak.rs`, `crates/grimoire/tests/pipeline_hint_leak.rs`, fixtures `hint-leaks-expected/` and `hint-leaks-ghost-got/`.
  - Input: the four rungs, the sample expected values, the curated hidden expected values and the ghost `got` values, all in memory. Flag any rung that contains one of those values as a JSON literal or a plain token, and any rung with a fenced block longer than 3 lines. Report the rung number and the value's index, never the value.
  - Requirements: PIP-14, PIP-46, P2.
  - Depends on: PA01.
  - Verify: both fixtures fail with `HNT-03`. A canary test asserts the leaked value is absent from the report text.
  - Rust lesson: borrowing `&[Value]` lets the check read secrets without taking ownership of them. Explain why the report is built from indexes and not from the values.

- [x] **PA06 [S] [SUPERVISED] (30 min) Ghost coverage, approval hash and publish checks (GHC-01, PUB-01, APR-01)**
  - Files: `crates/grimoire/src/pipeline/ghost_cover.rs`, `crates/grimoire/src/pipeline/approval.rs`, `crates/grimoire/tests/pipeline_ghost_approval.rs`, fixtures `one-ghost/`, `publish-without-approval/`, `approval-hash-mismatch/`, `ghost-not-reproducing/`.
  - `ghost_cover`: at least two known ghosts (GHC-01), and one verified S11 entry for each incorrect program (GHC-02, GHC-03 for the reference). `approval`: compare the assembled manifest hash with `approval.toml`, and check that a `published` status has an approval (`PUB-01` when missing, `APR-01` on mismatch).
  - Requirements: PIP-13, PIP-18, PIP-28, PIP-29, PIP-36, P1, P5.
  - Depends on: PA01.
  - Verify: each fixture fails with exactly its code. A test shows that editing one byte changes the hash and makes an existing approval stale.
  - Rust lesson: a hash is a fixed-size summary of bytes. Explain why comparing two hashes is safer and clearer than comparing file contents one by one.

---

## Wave 2: assembly and the S9 gate

- [x] **PA07 [S] [SUPERVISED] (30 min) Assembly script for the two trees**
  - Files: `scripts/pack-assemble.sh`, `scripts/tests/pack-assemble-test.sh`.
  - Usage: `scripts/pack-assemble.sh <slug> <public-root> <private-root>`. Copy the public files into `target/pack-assembly/<slug>/`, then the private files. Refuse if a private file name appears in the public tree, if the private tree is missing or unreadable (PIP-34), or if the slug directory names differ. Write the manifest (path and SHA-256 per file, sorted). Never print a hidden file's contents.
  - Requirements: PIP-31, PIP-34, PIP-32, D3.
  - Depends on: PA00. Uses the `problem-catalogue` C06 manifest format (same canonical bytes).
  - Verify: `pack-assemble-test.sh` runs with temporary directories: a normal assembly passes; a private file under the public root fails; a missing private root fails; output never includes a canary. The script is not run against the real private repo.
  - Rust lesson: none (shell). Record the shell safety choices used (`set -euo pipefail`, quoted paths).

- [ ] **PA08 [S] [BLOCKED on grimoire-packs amendment X1 and X2] (30 min) Teach grimoire the assembled layout**
  - Files: `crates/grimoire/src/layout.rs` (existing), `crates/grimoire/tests/layout.rs`.
  - Allow `hidden.toml`, `generator.toml`, `known_ghosts.toml`, `incorrect/*.py`, `reference/solution.py` and `approval.toml` in the assembled tree. Move the `[generator]` requirement to `generator.toml`. Add `LAY-05` for a file from the wrong tree.
  - Requirements: PIP-31, D3, design §4 and §14 X1.
  - Depends on: grimoire-packs amendment X1 and X2 (approved and specified there first). This task must not start on the old text.
  - Verify: a fixture with a `hidden.toml` in the public path fails with `LAY-05`; the same file in the assembled tree passes the layout check.
  - Rust lesson: a file allow-list is a set. Explain why adding a name to a set can widen what the validator accepts, and why each new name needs its own test.

- [ ] **PA09 [P] (30 min) Stage prompt templates S0 to S9**
  - Files: `.kiro/pipeline/prompts/S0.md` to `S9.md`.
  - Each template states: the stage's inputs, its output paths, its stop condition, its codes, and the checklist IDs it must answer. No template contains a retired value, a hidden value, a token or a path to the private credential. Each template asks for the author's own wording and forbids opening third-party problem text.
  - Requirements: PIP-6, PIP-41, PIP-3.
  - Depends on: PA00.
  - Verify: PA19 secret scan passes; PA10 checks that every checklist ID cited in these files exists.

- [x] **PA10 [P] (30 min) Reviewer checklist and completeness test**
  - Files: `.kiro/pipeline/checklists/reviewer.md`, `crates/grimoire/tests/pipeline_checklist.rs`.
  - Write the seven categories and the items from design §6, each with a unique ID `RV-<category>-<n>`. The test parses the file, checks that every category has at least one item, that IDs are unique, and that every `RV-*` ID cited in `.kiro/pipeline/prompts/` exists.
  - Requirements: PIP-21, PIP-47, PIP-46.
  - Depends on: PA00.
  - Verify: `pipeline_checklist` passes on the real file. A fixture checklist with a missing category fails the test.
  - Rust lesson: parsing text with `str::lines` and `split_once` is simple and explicit for a file this size. Explain why a full parser library would be more than the task needs.

- [x] **PA11 [P] (30 min) Author checklist and authoring skill**
  - Files: `.kiro/pipeline/checklists/author.md`, `.kiro/skills/grimoire-pack-authoring/SKILL.md`. Add a line for the skill in `.kiro/skills/SOURCES.md` marked original, MIT.
  - The skill covers: original wording (PIP-3), supported types (`int`, `str`, `list[int]`, `list[list[int]]`), statement rules (STM-03, "integer"), at least eight curated hidden cases, two or more realistic ghosts with hand-traced `got` values, four general hints (rung 4 is a near-solution step), `complexity_note` with time and space, and the self-review checklist to fill in.
  - Requirements: PIP-9 to PIP-15, PIP-3.
  - Depends on: PA09 (for the shared checklist IDs).
  - Verify: PA10 checklist-ID test covers the new author checklist too; the skill names only checks that exist in grimoire.
  - Rust lesson: none.

- [x] **PA12 [P] [SUPERVISED] (30 min) Reviewer agent definition and review skill**
  - Files: `.kiro/agents/pack-reviewer.json`, `.kiro/skills/pack-review/SKILL.md`.
  - The reviewer's tool list contains read tools and the pure validator (`validate_pack`), and no write tool and no code-running tool. Its prompt names the checklist and the output format (scores, findings with IDs and pointers, no quoted hidden values). It states that it has no author notes.
  - Requirements: PIP-2, PIP-22, PIP-38, PIP-41.
  - Depends on: PA10. Format of the agent JSON confirmed with the `custom-agent-creator` agent before editing.
  - Verify: a test reads the JSON and asserts that no write tool and no `run_reference`, `find_ghost_input` or `shrink_input` appears in its tools.
  - Rust lesson: none.

- [x] **PA13 [P] [SUPERVISED] (30 min) Author agent definition**
  - Files: `.kiro/agents/pack-author.json`.
  - Tools: read and write limited to one slug (by prompt instruction and by the publish guard), `validate_pack`, and the code-running tools only when a session is started by the operator with approval (D6). It must not have `status` write rights, `approval.toml` write rights, or write access to `docs/pack-reviews/`.
  - Requirements: PIP-1, PIP-2, PIP-39.
  - Depends on: PA12 (same definition format), PA16 (guard exists).
  - Verify: the same JSON test as PA12, extended to assert the author cannot write `docs/pack-reviews/`.
  - Rust lesson: none.

- [x] **PA14 [P] (25 min) One-page review summary template and an example**
  - Files: `.kiro/pipeline/templates/review-summary.md`, `docs/pack-reviews/README.md`.
  - Sections: what the problem asks (plain words), why it tests real skill, what the ghost shows, what could still be wrong, the seven scores in words, and the decisions Brighton must make. The template has no field for hidden values. Use `count-pairs` only as the example format, not as a review.
  - Requirements: PIP-27, PIP-42.
  - Depends on: PA00.
  - Verify: PA15's hidden-data scan passes on the template and README.

- [x] **PA15 [S] [SUPERVISED] (30 min) Review digest and routing script**
  - Files: `scripts/pack-review-digest.sh`, `scripts/tests/pack-review-digest-test.sh`.
  - Build the reviewer's input for one slug from the assembled tree, the checklist and the S9 to S11 reports. Write the output to a private location (`target/pack-review/<slug>/`, outside the repo) when hidden files are included. Write only the public-safe summary to `docs/pack-reviews/`. Run a scan (`SUM-01`) for any hidden pointer, canary or private file name in the summary.
  - Requirements: PIP-27, PIP-32, P2.
  - Depends on: PA07, PA14.
  - Verify: `pack-review-digest-test.sh` with a canary in a hidden fixture: the canary never appears in `docs/` output or in the summary; a summary that mentions `hidden.toml /case/3` fails `SUM-01`.
  - Rust lesson: none (shell).

---

## Wave 3: guards and credentials

- [x] **PA16 [S] [SUPERVISED] (30 min) Publish guard script and extended pack-validate hook**
  - Files: `scripts/kiro-hooks/pack-publish-guard.sh`, `scripts/tests/pack-publish-guard-test.sh`, and the extended `scripts/kiro-hooks/pack-validate.sh`.
  - The guard reads the hook's JSON on stdin. It blocks (exit 2 with a reason) a write that sets `status = "published"` in any `packs/*/meta.toml`, or that touches any `approval.toml`. It also extends the dangerous-shell guard so that `python`, `python3` or `node` on a path under `packs/` or `target/pack-assembly/` is refused. Allow ordinary edits.
  - Requirements: PIP-1, PIP-38, PIP-43, P1, P9.
  - Depends on: PA07.
  - Verify: shell fixtures for a published write (blocked), an `approval.toml` write (blocked), a `python3 packs/x/reference/solution.py` command (blocked), and an ordinary draft edit (allowed).
  - Rust lesson: none.

- [x] **PA17 [S] [SUPERVISED] (30 min) Pre-push check for `PUB-01`**
  - Files: `scripts/pre-push.sh` (extend), `scripts/tests/pre-push-pub-test.sh`.
  - For each commit being pushed, if a `packs/*/meta.toml` changes `status` to `published`, the commit message must contain "Approved by Brighton". Otherwise fail with `PUB-01`.
  - Requirements: PIP-28, PIP-44, P1.
  - Depends on: PA06 (code name), PA16.
  - Verify: a temporary git repository with two commits: one publishes without the phrase (refused), one publishes with it (allowed). Tests do not push anywhere.
  - Rust lesson: none.

- [x] **PA18 [S] [SUPERVISED] (20 min) Register the publish guard hook**
  - Files: `.kiro/hooks/pack-publish-guard.json` (created with the `createHook` tool, not by hand), `.kiro/hooks/pack-validate.json` (matcher widened to `packs/**` and the assembled tree).
  - Requirements: PIP-43.
  - Depends on: PA16.
  - Verify: the hook JSON parses; its command matches the script path; the matcher covers `packs/` and `fs_write|str_replace`.
  - Rust lesson: none.

- [ ] **PA19 [P] (25 min) Secret and prompt scan**
  - Files: `scripts/tests/pack-prompt-secret-scan-test.sh`.
  - Scan `.kiro/pipeline/`, `.kiro/agents/`, `.kiro/skills/grimoire-pack-authoring/`, `.kiro/skills/pack-review/`, `docs/pack-reviews/` and the scripts for token-like strings, `password=`, `PRIVATE_KEY`, `ghp_`, retired-value markers and any path to `.env`. Also check that `pack-batch.sh` and `pack-assemble.sh` never echo the credential variable.
  - Requirements: PIP-33, PIP-41, PIP-32.
  - Depends on: PA09, PA11, PA12, PA13.
  - Verify: the scan passes on the real files, and fails on a fixture containing `ghp_` followed by a fake token.
  - Rust lesson: none.

---

## Wave 4: credentials, backlog and rotation

- [ ] **PA20 [HUMAN] (25 min) Create the private repo, approve the originality list and access method**
  - Brighton creates `BrightonDube/counterspell-packs-private` (agents must not). He issues a read-only deploy key or a fine-grained read-only token and stores it outside the repository. He writes the originality list into the private repo. He decides open questions 3, 6 and 7.
  - Requirements: PIP-33, PIP-35 (prerequisite), PIP-5.
  - Depends on: PA00.
  - Verify: `docs/kiro-log/pack-pipeline-blockers.md` records the answers. The agent never sees the credential. No file in this repo changes.

- [ ] **PA21 [S] (30 min) Backlog: seven new ideas and the dry-run topic**
  - Files: `docs/pack-backlog.md`.
  - Seven original ideas that balance the tracks (arrays, strings, matrices) and include some medium problems. Each entry has: slug, track, concepts (must exist in `packs/catalogue.toml`), difficulty, a one-line original idea, and the planned ghost kinds. Include `lantern-streaks` as the dry-run topic. Do not name a third-party problem, and do not describe one.
  - Requirements: PIP-7, PIP-3, PIP-4.
  - Depends on: PA20 (originality list exists).
  - Verify: a check that every concept and track in the backlog exists in `packs/catalogue.toml` (TAX-01); every slug is unused (SLUG-01).

- [ ] **PA22 [S] [SUPERVISED] (25 min) Rotation procedure for the three starter packs (plan only)**
  - Files: `.kiro/pipeline/rotation.md`.
  - Written procedure: record the retired canonical values as SHA-256 hashes in the private `retired-values.toml` (PIP-35); write new `hidden.toml` and `generator.toml` in the private tree; bump `pack_version` to 2; re-run S5 to S13. Do not run it in this task. The starter packs are `draft`.
  - Requirements: PIP-35, D10.
  - Depends on: PA06 (HID-03 retired-hash check exists in the grimoire amendment), PA20.
  - Verify: the procedure lists each of the three packs and each step. A reviewer checks it against PIP-35.

---

## Wave 5: dry run and batch mode

- [ ] **PA23 [S] (30 min) Dry-run harness with a fake `PackRunner`**
  - Files: `crates/grimoire/tests/pipeline_dry_run.rs`, `crates/grimoire/tests/fixtures/pipeline/fake_runner.rs` (test double only).
  - The fake runner returns recorded outputs for the fixture programs, and it is labelled in its file header as not evidence that the sandbox works (`tdd-and-quality` rule on test doubles). It implements the `PackRunner` trait from grimoire T17.
  - Requirements: PIP-17, PIP-48, PIP-49.
  - Depends on: PA01, PA06.
  - Verify: the harness compiles and runs one fixture pack through S9 with the fake runner in a temporary directory.
  - Rust lesson: a trait is a contract. A test double that implements the same trait lets you test the pipeline without the real runner. Explain why the double must say what it is not.

- [ ] **PA24 [S] [SUPERVISED] (30 min) Dry run on `lantern-streaks`**
  - Files: `crates/grimoire/tests/pipeline_dry_run.rs` (extend), `crates/grimoire/tests/fixtures/pipeline/lantern-streaks/` (fixture pack, not in `packs/`).
  - Run S0 to S9 for the fixture, with the fake runner, in a temporary directory. Assert: S9 passes; S10 and S11 are `pending_judge`; the state file has no hidden value (canary check); `git status` in the repository is unchanged.
  - Requirements: PIP-48, P2, P3.
  - Depends on: PA23, PA15, PA16.
  - Verify: the test passes; the report shows the stage reached and the codes, never the hidden values.
  - Rust lesson: none beyond PA23.

- [ ] **PA25 [S] [SUPERVISED] (30 min) Batch orchestrator `scripts/pack-batch.sh`**
  - Files: `scripts/pack-batch.sh`, `docs/kiro-log/pack-batch-<date>.md` (written at run time).
  - Implements design §11: per slug S0 to S9 headless; stop at S10 and S11 with `static_complete`; review; on pass, S13 summary and pull request through `gh` in draft-safe mode; never `published`, never `approval.toml`, never the importer; the loop cap of three; a log with codes and counts only.
  - Requirements: PIP-4, PIP-1, PIP-20, PIP-25, PIP-39.
  - Depends on: PA03, PA07, PA15, PA16, PA21, PA24.
  - Verify: covered by PA26. Shell syntax check (`bash -n`) passes.
  - Rust lesson: none.

- [ ] **PA26 [S] [SUPERVISED] (30 min) Batch tests with recorded outputs**
  - Files: `scripts/tests/pack-batch-test.sh`.
  - Run the batch on two fixture slugs with recorded S10 and S11 reports and a fake reviewer that fails twice then passes. Assert: the state reaches `awaiting_approval`; no `published` appears in any file; no `approval.toml` is written; the importer is not run; a fixture that always fails stops at round 3 with `escalated`.
  - Requirements: P8, PIP-25, PIP-49.
  - Depends on: PA25.
  - Verify: `pack-batch-test.sh` passes against temporary directories, and the test count of the directory is recorded.
  - Rust lesson: none.

---

## Wave 6: operator runs, approval and handover

- [ ] **PA27 [HUMAN] [BLOCKED on judge-sandbox T20 to T22, T02, T37 and grimoire-packs T20] (30 min per pack) Sandbox and ghost checks on the host**
  - For each pack in the launch set, the operator runs S10 (`SBX-01`, `SBX-02`, `SBX-09`, `SBX-03`) and S11 (`find_ghost_input` on each incorrect program, and "not found" on the reference) through the judge's sandbox. The report records the sandbox image digest. Brighton approves the code-running tools for that session (PIP-39).
  - Requirements: PIP-17, PIP-18, PIP-39.
  - Depends on: PA25, the judge `PackRunner`.
  - Verify: each pack's state moves from `static_complete` to `sandbox_checked`, with the digest recorded. Until then, the pack is not reviewed as passed.

- [ ] **PA28 [HUMAN] (15 min per pack) Brighton's approval gate**
  - Brighton reads `docs/pack-reviews/<slug>.md` and the private report, merges the pull request after editing `status` to `published` in the approval commit ("Approved by Brighton"), and writes `approval.toml` in the private repo with the content hash. Any rejection goes back to the findings with a new round.
  - Requirements: PIP-28, PIP-20, PIP-29.
  - Depends on: PA27 for the pack.
  - Verify: `PUB-01` passes on the approval commit; the importer (`problem-catalogue`) accepts the hash; the importer refuses a changed file (APR-01). Import itself is run by Brighton, not in this task.

- [ ] **PA29 [S] (25 min) Repo-wide gates for the pipeline**
  - Run the Rust gate for `counterspell-grimoire` (fmt, clippy with `-D warnings`, tests), `bash -n` on every new script, the shell test scripts, and the workspace `cargo test --workspace`. Record the test count before and after. Check that no test was removed (`tdd-and-quality` test-integrity rules).
  - Requirements: PIP-50, PIP-49.
  - Depends on: PA01 to PA26 (those that apply).
  - Verify: each command's exit code and the count are written to the task notes.
  - Rust lesson: `cargo clippy -D warnings` makes warnings errors, which keeps the rules from drifting. Explain why the test count matters when a gate is green.

- [ ] **PA30 [S] (25 min) Operator guide for the pipeline**
  - Files: `docs/pack-pipeline.md`.
  - One page for Brighton, in plain English: how a topic becomes a pack, which steps he runs, what he approves, how to read the summary, what a `draft` pack means, how to reject, and how the split between public and private works. No hidden values, no credentials.
  - Requirements: PIP-27, PIP-31, PIP-33.
  - Depends on: PA28 (after the first pack has been approved, so the guide describes real steps).
  - Verify: PA19 secret scan passes on the guide; PA15's summary scan passes.

---

## Traceability

| Task | Requirements |
|---|---|
| PA00 | all (approval) |
| PA01 to PA06 | PIP-3, PIP-8, PIP-13, PIP-14, PIP-18, PIP-22 to PIP-29, PIP-36, PIP-45, PIP-46 |
| PA07, PA15, PA16, PA17, PA18, PA19 | PIP-31 to PIP-34, PIP-38, PIP-41, PIP-43, PIP-44 |
| PA08 | PIP-31, design §14 X1, X2 |
| PA09 to PA14 | PIP-2, PIP-3, PIP-6, PIP-9 to PIP-15, PIP-21, PIP-22, PIP-27, PIP-42 |
| PA20 to PA22 | PIP-3, PIP-7, PIP-33, PIP-35 |
| PA23 to PA26 | PIP-1, PIP-4, PIP-17, PIP-20, PIP-25, PIP-39, PIP-48, PIP-49 |
| PA27 to PA30 | PIP-17, PIP-18, PIP-20, PIP-27, PIP-28, PIP-29, PIP-31, PIP-33, PIP-39 |

## Blocked and deferred items (for the blockers file)

- PA08: waits for grimoire-packs amendments X1 and X2. Those are not in grimoire-packs `tasks.md` yet.
- PA27: waits for judge-sandbox T20 to T22, T02 and T37 (VPS), and grimoire-packs T20.
- PA28 for the starter packs: waits for PA22 (rotation) and PA27.
- `problem-catalogue` import (C14, C17) must check `approval.toml` before any starter pack can be approved (amendment X3).
- Rotation of the starter packs is deferred until Brighton approves PA22 and creates the private repo (PA20).
- Deferred to later specs: `reference_complexity` and `[bench]` for the launch set (efficiency-familiar T40, with a version bump, X7).
- Deferred: estimated completion time, learning objectives, prerequisites and an editorial field for packs (`33-starter-pack-hardening-v2.md`, "deferred").
