# pack-batch run 2026-11-25

Batch 01 — ten original packs on the arrays track (completion-program CP16).

## S9 static validation (counterspell-grimoire validate --deny-warnings)

- lantern-streaks: 0 errors, 0 warnings
- tide-table: 0 errors, 0 warnings
- candle-frequency: 0 errors, 0 warnings
- twin-echoes: 0 errors, 0 warnings
- sigil-groups: 0 errors, 0 warnings
- prefix-echoes: 0 errors, 0 warnings
- twin-ledgers: 0 errors, 0 warnings
- harvest-tally: 0 errors, 0 warnings
- warden-sums: 0 errors, 0 warnings
- equal-embers: 0 errors, 0 warnings

Review fixes during the round: KNG-05 concept-tag gaps closed on
tide-table, twin-ledgers and warden-sums; a KNG-04 minimality phrase
reworded on candle-frequency; empty-input edges stated in words on six
statements (RV-clarity-3); a `prefixes` tag added to equal-embers.

## S10/S11 sandbox run (sbx_batch01, real rootless daemon)

Daemon `unix:///run/user/1000/docker.sock`, identity-verified rootless.
Image `python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90`.
`cargo test -p counterspell-catalogue-cli --features sandbox --test
sbx_batch01 -- --ignored --test-threads=1` — 1 passed.

| slug | S10 fill | S11 ghosts | S11 reference |
|---|---|---|---|
| lantern-streaks | Filled | count-all-lit, first-streak found | not-found |
| tide-table | Filled | earlier-hour, non-increase found | not-found |
| candle-frequency | Filled | first-seen-wins, return-the-count found | not-found |
| twin-echoes | Filled | counts-every-repeat, needs-three found | not-found |
| sigil-groups | Filled | group-by-value, return-sigil-value found | not-found |
| prefix-echoes | Filled | rescan-whole-list, short-slice found | not-found |
| twin-ledgers | Filled | join-by-position, overwrite-last found | not-found |
| harvest-tally | Filled | every-position-pair, ordered-pairs found | not-found |
| warden-sums | Filled | only-from-start, stop-at-quota found | not-found |
| equal-embers | Filled | stop-at-first-balance, update-every-visit found | not-found |

## Review round

One review round per pack; every category scored strong or solid, no
blocker or unresolved finding. Public summaries in `docs/pack-reviews/`
passed the SUM-01 scan. Secret scan over the ten public trees clean.

## Publication

All ten approved under the delegated publication instruction of
21 November 2026: `status = "published"` in this commit; `approval.toml`
per slug lands in the private repo stamped with this commit's hash.

## Import and publish (production catalogue)

`counterspell-catalogue import --packs <public> --private-root <private>
--deny-warnings` ran on the prod host inside the server image against the
prod rootless sandbox daemon (`/run/user/1002/docker.sock`). All ten
batch-01 packs reported `created version=1`; `lantern-ledger` and
`mirror-signal` reported `unchanged`. Each slug then went through
`publish` (`created version=1`).

Approval-hash correction: the first approval stamps used a recursive
file walk that included `fixtures/`; `build_manifest` covers top-level
files plus `reference/` and `incorrect/` only. All thirteen stamped
approvals were re-recorded against the correct canonical hash — approved
content unchanged (private PR #6).

Pre-existing, out of batch-01 scope: the three rotated starters
(count-pairs, diagonal-total, longest-run-length) refuse
`version_not_bumped` on re-import — their rotated private content
differs from the stored version without a `pack_version` bump. They
remain published at their stored versions; a proper bump needs its own
review-and-approval round.

## Production verification (B01-07)

- `GET /api/problems` serves all 15 problems including the ten new
  slugs with correct tracks, difficulties and concepts.
- `GET /api/problems/<slug>` 200s for the new slugs.
- One real production submit path verified end-to-end: an ops-verify
  account (handle `ops-verify-cp16`, minted session, revoked after)
  POSTed `mode=submit` for `lantern-streaks` in Python. Run
  `aa3a0a0b-8442-4e2f-bcd1-0f4f5600c995` completed `accepted`,
  27/27 (3 samples + 24 hidden), ~1s through the prod rootless
  sandbox. The session row was deleted after the check.

Codes and counts only - no hidden values are logged.
