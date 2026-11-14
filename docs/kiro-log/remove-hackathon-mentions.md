# Remove event-specific wording (session 37)

Started 14 Nov 2026. Brighton approved this change (14 Nov 2026). Prompt: `docs/kiro-prompts/37-remove-hackathon-mentions.md`.

## Inventory before the change

Command (from the prompt), run at the start of this session:

```
rg -i --hidden -g '!.git/**' -g '!**/node_modules/**' -g '!**/target/**' -g '!**/.venv/**' -g '!**/dist/**' -e 'hackathon' -e 'devpost' -e 'kiroween'
```

- Measured now: 141 matches on 116 lines in 22 files (counted with `rg -o`, `rg`, and `rg -l`).
- The prompt's 14 Nov figure was 101 matches on 90 lines in 15 files. The gap is because the session logs grew since then and the counts were taken differently. This plan uses the current measurement.
- Also in the list: `Costume Contest`, `Judging?`, `judge demo account`, `Devpost private field`, `hackathon judge`, `hackathon build`, `hackathon requires`, `hackathon credits`.

## Decisions

Keep (allowed by the prompt or by policy):
- `.kiro/steering/brighton-decisions.md`: the policy bullets that forbid the event name.
- `docs/kiro-prompts/37-remove-hackathon-mentions.md`: this prompt. Its filename also stays.
- `scripts/kiro-run.sh`, `scripts/kiro-implement-all.sh`, `scripts/kiro-pending-spec-amendments.sh`, and the `usage()` function in `scripts/kiro-gap-specs.sh`: the vendor pool regex `kiroween-2026:` that parses `/usage` output.
- `scripts/kiro-gap-specs.sh` and `scripts/kiro-after-current.sh`: the queue references to this prompt's filename, the log filename and the session label. Renaming them would break the queue.
- `docs/kiro-playbook.md`: process doc for the external submission. Allowed to keep its mentions.
- `docs/kiro-log/` historical transcripts and logs. Not edited.
- `docs/KIRO_USAGE.md` line 1 (the write-up header) is reworded. Its log entries change only the phrase "hackathon credits" to "credits". Numbers stay the same.
- `docs/system-design.md` line with `kiroween.devpost.com/rules`: the official rules URL is licence evidence. The URL stays. Its label changes to "Official competition rules".

Edit:
- `.gitignore`: comment `(hackathon rule)` becomes `(repository rule)`.
- `scripts/kiro-run.sh`: the log line `hackathon credits used now` becomes `credits used now`. Nothing parses that wording.
- `docs/kiro-prompts/09-ide-workspace*.md`: "Costume Contest showcase" becomes "showcase". These are prompt sources, not logs.
- `docs/requirements.md`, `docs/system-design.md`, `docs/product-brief.md`, `docs/licence-rules.md`: rewords. Dates, the demo account, uptime until 29 Jan 2027, feature freeze and the video stay.
- `.kiro/specs/auth/*`, `.kiro/specs/ide-workspace/requirements.md`, `.kiro/specs/counterspell-mcp-power/requirements.md`: rewords. "judge demo account" becomes "demo account". "Devpost private field" becomes "private field of the external submission form".
- `web/src/routes/login/+page.svelte`: link text "Judging? Use the demo account" becomes "Try the demo account". No test asserts the old text (checked with `rg` before the change).
- Rust doc comments in `crates/server` that say "judge demo account" become "demo account". Comments only. The trusted `counterspell-judge` worker keeps its name.

## Plan (small commits)

1. Scripts, `.gitignore` and prompt-source comments.
2. Rust and web wording (comments and the login link text), with the test gate.
3. Specs (`.kiro/specs/`).
4. Docs: requirements, system design, product brief, licence rules.
5. `docs/KIRO_USAGE.md` phrase change.
6. Full gate, `rg` inventory again, this log updated, final commit.

## Checks

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cd web && npm run check && npm run build` (and `npm run test` if the script exists)
- No `.env` or secret file read or written. No push, no branch change.

## Results

Counts use the same `rg` command (`-o` for matches, plain for lines, `-l` for files).

| Measure | Before (start of run) | After |
|---|---|---|
| Matches | 141 | 76 |
| Lines | 116 | 47 |
| Files | 22 | 14 |

Remaining hits, by category (all allowed):
- Policy bullets: `.kiro/steering/brighton-decisions.md` (line 18).
- This prompt and its filename: `docs/kiro-prompts/37-remove-hackathon-mentions.md`.
- Usage-pool regex `kiroween-2026:` and its `usage()` function: `scripts/kiro-run.sh`, `scripts/kiro-implement-all.sh`, `scripts/kiro-pending-spec-amendments.sh`, `scripts/kiro-gap-specs.sh`.
- Queue references to this session's prompt filename and log label: `scripts/kiro-gap-specs.sh` (lines 4, 57, 61 to 66), `scripts/kiro-after-current.sh` (line 26). Renaming them would break the queue.
- Official rules URL, kept as licence evidence: `docs/system-design.md` line 641.
- External process doc: `docs/kiro-playbook.md` (allowed by the prompt).
- Historical logs, not edited: `docs/kiro-log/spec-gaps-2026-11-14.md`, the deploy-ops and problem-catalogue spec logs, and this session's own run log.
- This plan file.

Not changed on purpose:
- The word "judge" where it means the trusted `counterspell-judge` worker or "judging" as product behaviour. That is product terminology, not the event.
- Persona name "Judge J" in `docs/requirements.md`. It is an invented example name.
- The `Judging?` link text was reworded to "Try the demo account" in the login page, the auth spec and tasks. No test asserted the old text.
- The original scoring criteria line in `docs/product-brief.md` was removed from the brief. The text remains in git history.

## Gate results (fresh runs after all edits)

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass, no warnings.
- `cargo test --workspace`: all suites pass, exit 0. `pg_`/`valkey_` tests are ignored by convention and were not run.
- `cd web && npm run check`: 0 errors, 0 warnings.
- `cd web && npm run build`: pass, site written to `build/`.
- `cd web && npm run test`: 39 test files passed.

Test counts did not drop: no test was removed, skipped or weakened. Only comments and one UI label changed.

## Commits (on the current branch, not pushed)

- `e723c57` chore: neutral wording in run script, gitignore and prompt sources
- `b82954a` chore: neutral demo account wording in server comments and login link
- `2b299bb` docs(specs): neutral demo account and reviewer wording
- `2c1a102` docs: replace event-specific wording with neutral terms
- `61e0741` docs: drop event name from usage log wording
- this plan file: committed as the final commit of this run

The run log `docs/kiro-log/2026-11-14T2329-37-remove-hackathon-mentions.log` is written by `scripts/kiro-run.sh`. It is not committed here because the queue script commits it.

