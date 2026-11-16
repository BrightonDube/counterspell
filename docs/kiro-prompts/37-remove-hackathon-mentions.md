# 37 - Remove event-specific wording from product artefacts

Added 14 Nov 2026. This is a default-mode implementation session, not a new spec. Approved by Brighton (14 Nov 2026).

Follow AGENTS.md and every file in `.kiro/steering/` (read `brighton-decisions.md` first, especially the 14 Nov decisions about event wording and about hidden tests staying in this repo). Strict TDD: `.kiro/steering/tdd-and-quality.md` and the `test-driven-development` skill. Run the full test suite and lints after the change, then commit. Do not push and do not change branches. Never start another Kiro session.

## Goal

Find every mention of the hackathon, Devpost, or the event name (Kiroween / kiroween) in code, UI strings, comments, config, tests, specs and steering, and reword or remove it neutrally without changing behaviour.

Inventory first (and again at the end):

```
rg -i --hidden -g '!.git/**' -g '!**/node_modules/**' -g '!**/target/**' -g '!**/.venv/**' -g '!**/dist/**' -e 'hackathon' -e 'devpost' -e 'kiroween'
```

Also search close variants that only exist because of the event (for example "Costume Contest" as a category label, "Devpost private field", "hackathon credits", "hackathon rule", "hackathon build", "hackathon judge", "hackathon requires") and reword those the same way.

Use neutral wording such as "launch", "v1 deadline", "credits", "demo account", or "external submission form". Keep real dates and product requirements (the demo account, uptime until 29 Jan 2027, feature freeze, the short video). Do not change behaviour, schemas, routes, test assertions about product behaviour, credit-pool parsing, or CI.

A pre-change inventory on 14 Nov 2026 found 101 matches on 90 lines in 15 files. Most hits are the repeated "hackathon credits used now" line in `docs/KIRO_USAGE.md` (from `scripts/kiro-run.sh`), plus `docs/requirements.md`, `docs/system-design.md`, `docs/kiro-playbook.md`, licence rules, a few specs, scripts, and a `.gitignore` comment.

## Keep

- Genuine external or submission docs under `docs/` that truly need the event name. `docs/kiro-playbook.md` is a process doc for the event and may keep those mentions. A future submission write-up would also stay. Do not create a write-up; Brighton's assistant does that, and it is not a spec.
- The standing decision bullets in `.kiro/steering/brighton-decisions.md` (they name the event in order to forbid naming it elsewhere). Do not delete those policy bullets or rephrase them so the decision disappears.
- This prompt file, and any queue-script comment that only names this session.
- The vendor credit-pool name in `kiro-cli /usage` output. Scripts parse `kiroween-2026:` from that output in `scripts/kiro-run.sh`, `scripts/kiro-implement-all.sh` and `scripts/kiro-gap-specs.sh`. Keep the regex that matches the CLI's pool name, or remaining-credit checks will break. You may neutralize comments and the "hackathon credits used now" log wording (for example "credits used now %s/4000").
- Historical session transcripts under `docs/kiro-log/` that merely quote past runs. Leave those alone unless a mention sits in a living source of truth (specs, requirements, system design, steering, scripts, comments, UI).
- In `docs/KIRO_USAGE.md`, you may replace the repeated phrase "hackathon credits" with "credits" and leave the numbers unchanged. Do not otherwise rewrite the usage log. `scripts/kiro-run.sh` must append a neutralized line from now on.

## Do not

- Leave the event named in code, UI text, comments, config, tests or specs after you are done, except the usage-pool regex and the policy bullets above.
- Change product behaviour, add features, or start any other spec.
- Push, switch branches, or start another Kiro session.
- Touch running work you do not need to edit. Leave unrelated dirty files alone. Commit only the files you changed.
- Weaken tests. If a test string mentions the event, change that string and the production string together so the test still checks the same behaviour.
- Spend money, deploy, or edit `.env`.

## TDD and finish

This is mostly wording. You do not need a red test for a comment-only change. You do need:

1. Inventory and a short plan, written to `docs/kiro-log/remove-hackathon-mentions.md` before you start editing.
2. Edit in small commits (for example: scripts and comments; specs; `docs/requirements.md` and `docs/system-design.md`; UI strings if any).
3. After the changes, run the full relevant suite fresh: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and for web `cd web && npm run check && npm run build`, plus `npm run test` if that script exists. Fix every failure before the last commit.
4. Re-run the `rg` inventory. Confirm remaining hits are only the allowed set (policy bullets, this prompt, genuine external docs, the usage-pool regex, and leftover historical log phrases you chose to leave). Append the before/after counts and the remaining hits to `docs/kiro-log/remove-hackathon-mentions.md`.
5. Commit. Do not push.

Finish with a summary: files changed, remaining `rg` hits, test results, and commit hashes.
