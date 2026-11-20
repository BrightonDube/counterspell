# pack-authoring-pipeline blockers and approvals

## PA00 approval — 15 Nov 2026 10:26 (Brighton)

Brighton approved this spec for implementation. Details are also in `.kiro/steering/brighton-decisions.md`.

Limits that override assumed choices where they conflict:

- Paid services and any other spending stay held. Do not buy, subscribe, or call a paid API as a paid customer.
- The private pack repo is still waiting on Brighton (same as problem-catalogue C00a). Tasks that need that repo stay unchecked until he creates it.
- Other open questions 1 to 16 keep their assumed choices, except as above. Stop only for a serious security, licensing-risk, cost or production-data decision that is not already settled.

## Assumed choices made during implementation (not blockers)

- **PA04, S1 record headings.** The requirements (PIP-8) name the record's content but not five headings. The guard requires these level-two headings, matched case-insensitively, each with some text: `Idea`, `Why it is original`, `Patterns used`, `Checklist answers`, `Third-party text avoided`. The fifth heading is the assumed choice, because PIP-3 requires the author to avoid third-party text. Brighton can rename them; the constant is `REQUIRED_SECTIONS` in `crates/grimoire/src/pipeline/originality.rs`.
- **PA06, hashing.** `counterspell-grimoire` does not compute SHA-256. Adding a hashing crate is not named in PA06, so the crate stays as it is. The manifest hash comes from `sha256sum` in `scripts/pack-assemble.sh` (PA07). The "one edited byte changes the hash" check moves to the assembly script test (PA07). PA06 tests the comparison: a missing approval gives `PUB-01`, a different hash gives `APR-01`, and an approval made before an edit becomes stale.

## PA19 and PA09 (secret and prompt scan, stage prompts)

- `scripts/pack-secret-scan.sh` is the scanner. It reports file, line and code, never the matched text. Its six codes are SEC-GH, SEC-KEY, SEC-PW, SEC-ENV, SEC-ECHO and RET-01.
- Assumed choice: the scan does not flag the names `retired-values.toml`, `hidden.toml` and similar. Scripts must name them to refuse them, and a file name is not a secret. The scan flags the retired marker pattern (`RETIRED-` followed by a capital or digit) instead.
- Assumed choice: the scan checks the pack scripts (`scripts/pack-*.sh`, `scripts/kiro-hooks/pack-*.sh`) and not every script in `scripts/`. Older scripts already contain regexes that name key prefixes, which would be false positives.
- `pack-batch.sh` (PA25) does not exist yet. When it is written, run `scripts/pack-secret-scan.sh scripts/pack-batch.sh` against it.

## Blocked and deferred tasks (this run)

- **PA08** (teach grimoire the assembled layout). Blocked on grimoire-packs amendments X1 and X2. Those are not specified in `grimoire-packs` yet. The current allow-list in `crates/grimoire/src/pack.rs` rejects `hidden.toml`, `generator.toml` and `approval.toml`, and `meta.toml` still needs `[generator]`. Unblock: Brighton approves X1 and X2 and a grimoire-packs task adds the layout.
- **PA20** (`[HUMAN]`, private repo, originality list, access method). Brighton created the private repo and the read-only deploy key on 15 Nov 2026 10:45 (`brighton-decisions.md`). Still open for Brighton: the originality list, and open questions 3, 6 and 7. PA20 stays unticked until he records those answers.
- **PA21** (seven backlog ideas). Depends on PA20 (the originality list). Not started. Unblock: PA20.
- **PA22** is written as a plan, but not ticked, because its check is a reviewer's (see the review queue).
- **PA23 and PA24** (fake-runner dry run, `lantern-streaks` dry run). The S9 stage needs a layout that accepts the assembled tree, which PA08 provides. Without PA08, S9 cannot pass. Unblock: PA08.
- **PA25 and PA26** (batch orchestrator and its tests). Depend on PA21 (backlog) and PA24 (dry run). Unblock: PA21 and PA24.
- **PA27** (`[HUMAN]`, sandbox and ghost checks on the host). Depends on judge-sandbox T20 to T22, T02 and T37 (VPS), and grimoire-packs T20. Not run.
- **PA28** (`[HUMAN]`, Brighton's approval gate). Not run.
- **PA30** (operator guide). Depends on PA28. Not written.
- **Batch mode and the importer.** `problem-catalogue` C14 and C17 (the importer's approval check, amendment X3) are not implemented, so PIP-36 is not enforced by the importer yet.

## PA29 gate record (this run)

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0. 1218 passed, 0 failed, 331 ignored (the existing `pg_`, `valkey_` and `sbx_` ignored tests).
- `cargo test -p counterspell-grimoire`: exit 0, and the count rises with each task.
- `bash -n` on every new pack script, the hook scripts and the shell tests: exit 0.
- Shell tests: assemble 17, digest 7, publish guard 10, pre-push publish 4, hook registration 6, secret scan 7. All pass.
- Test integrity: `git diff f54c631 HEAD` shows 68 `#[test]` lines added and 0 removed in `crates/`.
- PA29 covers the completed tasks only. PA08, PA20 to PA28 and PA30 are open (see the blocked list above).

## Final gate for this run (after the end-of-wave review fixes)

- `cargo fmt --all -- --check`: exit 0. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace`: exit 0, 1220 passed, 0 failed, 331 ignored (were 1218 at PA29).
- Shell tests, all exit 0: hooks-register 6, pack-assemble 18, pack-prompt-secret-scan 7, pack-publish-guard 17, pack-review-digest 9, pre-push-pub 4.
- Test integrity: 70 `#[test]` lines added in `crates/` and `scripts/` since the run started, and 0 removed.

## Decisions for Brighton (open)

1. **Approval identity (C2).** The "Approved by Brighton" phrase does not prove who approved. Decide whether the publish gate should require a signed commit from Brighton's key, or an allow-listed author.
2. **Short hidden values.** The summary scan skips values under four characters. Decide whether a reviewer check is enough, or whether the scan should refuse them too.
3. **Author prompt.** The author may not write `docs/pack-reviews/`, but only the prompt forbids it, because no hook blocks that path yet. Decide whether to add a path guard.

## Resume run, 16 Nov 2026 (attempt 2): nothing new to tick

Checked each unticked task against its dependencies and the blockers above. No task could be completed in this run without a decision only Brighton can make.

- **PA08:** still blocked. `.kiro/specs/grimoire-packs/` has no amendment X1 or X2 (checked with grep on 16 Nov 2026). The design's §14 lists them as requested, not approved.
- **PA23 and PA24:** their own dependencies (PA01, PA06) are done. Their verification still needs an S9 pass on an assembled tree, which needs PA08. There is no stage runner module in `crates/grimoire/src/pipeline/` yet. Left unticked.
- **PA20 (`[HUMAN]`):** open for Brighton. The private repo and deploy key are in place (15 Nov 10:45). The originality list and open questions 3, 6 and 7 are still his to answer.
- **PA21 and PA22:** PA21 depends on PA20. PA22 has a plan file (`.kiro/pipeline/rotation.md`, commit 5036f45) and stays unticked, because its check is a reviewer's check against PIP-35.
- **PA25 and PA26:** depend on PA21 and PA24.
- **PA27, PA28, PA30:** `[HUMAN]` or dependent on them. Not run.

Unblock order for Brighton: approve grimoire-packs X1 and X2 (then PA08 runs), answer PA20 (then PA21 and PA22 can be checked), and decide the three open items in "Decisions for Brighton" above.

## Unblock decisions, 20 Nov 2026 (Brighton, in chat)

- **PA08 (X1/X2).** Brighton approved the drafted amendment with edits; the approved
  text is recorded in `.kiro/specs/grimoire-packs/tasks.md` under "Amendment X1 + X2
  (approved by Brighton, 2026-11-20)". PA08 proceeds.
- **PA20 open questions.** Brighton confirmed the assumed answers: Q3 = private repo
  `BrightonDube/counterspell-packs-private` with a read-only deploy key kept outside the
  repo (already in place since 15 Nov); Q6 = the originality list lives in the private
  repo (drafted from `docs/licence-rules.md` in this session, Brighton to review);
  Q7 = authoring agents do not open third-party problem statements.
- **PA27.** Brighton approved the code-running tools for this session (PIP-39): the
  agent runs S10 and S11 for the three launch packs through the real rootless sandbox
  and records the image digests in each pack's state.
- **PA28.** The launch packs are `published`; the approval commit and `approval.toml`
  were confirmed by Brighton on 18 Nov and the importer accepts the recorded hashes
  (verified 20 Nov). The task's verify legs are met.
- **Grimoire T29.** Brighton's call on design open question 11: convert `packs/_test/`
  to the two-tree format now (ghost-engine has shipped).
- **Catalogue C40.** Brighton chose full rotation: every hidden value published in
  public git history at grimoire T21–T23 (known-ghost inputs/got values, the reference
  and the incorrect programs) is retired; all three packs get new hidden cases, a new
  generator seed and a `pack_version` bump. Open question 21: do NOT rewrite public
  git history — rotation invalidates the leaked values.
- **MCP T16/T17.** The agent runs the scripted find/fix/re-verify session through
  kiro-cli and files the transcript under `docs/kiro-log/chats/`; screenshot captures
  for T17 remain Brighton's leg in the Kiro IDE.
- **Deploy T23–T32.** The shared-host path superseded the dedicated-VPS order: the host
  (15.204.92.66, shared with Evorayn) answers SSH with publickey-only auth, Caddy is
  live, the four `production` environment secrets and `DEPLOY_ENABLED=true` are set, and
  Deploy runs queue behind the environment's required reviewers. Remote evidence is
  recorded in `deploy/vps/verification-report.md`; legs that need an on-host session
  (rootless facts, env file modes, Postgres roles, backup/restore, monitors, rollback
  rehearsal) stay marked as recorded there — nothing is claimed without evidence.

## PA20 — open questions confirmed by Brighton (2026-11-20)

Brighton confirmed all three assumed answers:

- **Q3 (private repo name and access).** `BrightonDube/counterspell-packs-private`,
  created by Brighton, accessed by a read-only deploy key scoped to that repo
  and stored outside the public working tree. Done 15 Nov.
- **Q6 (originality list).** A list of synthetic markers and phrases the author
  must not reuse, kept in the private repo; the list itself contains no
  third-party problem text. Drafted as `originality-markers.txt` in the private
  repo — Brighton reviews the contents before the first pipeline run (his
  approval is the PA20 gate).
- **Q7 (third-party reading).** Authoring agents never open third-party problem
  statements; they may read `docs/licence-rules.md` and the spec only.

## REF-01 severity reconciliation (2026-11-20, needs Brighton's awareness)

Amendment X1 (approved) says a missing reference in the public-only tree is
`REF-01` at severity warning. T27 (approved) runs CI with
`validate --all packs --public --deny-warnings`, under which every warning is a
failure — so a literal REF-01 warning would fail all three catalogue packs on
every CI run. Implementation reconciles the two approvals: in public mode the
absent reference marks the stage `skipped` in the report (visible, non-fatal);
in assembled/pair mode a missing reference remains the REF-01 error. If a
literal public-mode warning is wanted instead, the CI flag must be relaxed.
