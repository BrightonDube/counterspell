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

# pack-batch run 2026-11-25 (batch 02)

Batch 02 — ten original packs on the arrays track, two-pointer and
sequence-scan theme (completion-program CP16): sigil-dedup, ember-drift,
twin-tides, candle-compact, harbour-depths, sorted-embers, shared-sigils,
closest-quota, triad-rite, flood-line.

## S9 static validation (validate <pack> --deny-warnings)

All ten assembled trees report 0 errors, 0 warnings. Review fixes during
the round: rung-3 code fences trimmed to three lines on seven packs
(HNT-02), concept tags completed on harbour-depths, flood-line and
triad-rite (KNG-05), a banned minimality phrase reworded on
closest-quota (KNG-02), and one schema-out-of-range ghost input
corrected (KNG-02). Review round 2: `sorted = true` added to every
ordered array in the six sorted-input packs (Devin finding on PR #129);
validation and the sandbox suite re-ran clean.

## S10/S11 sandbox run (sbx_batch02, real rootless daemon)

Daemon `unix:///run/user/1000/docker.sock`, identity-verified rootless.
Image `python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90`.
`cargo test -p counterspell-catalogue-cli --features sandbox --test
sbx_batch02 -- --ignored --test-threads=1` — 1 passed (~45s).

| slug | S10 fill | S11 ghosts | S11 reference |
|---|---|---|---|
| sigil-dedup | filled | 2 counterexamples found | not-found |
| ember-drift | filled | 2 counterexamples found | not-found |
| twin-tides | filled | 2 counterexamples found | not-found |
| candle-compact | filled | 2 counterexamples found | not-found |
| harbour-depths | filled | 2 counterexamples found | not-found |
| sorted-embers | filled | 2 counterexamples found | not-found |
| shared-sigils | filled | 2 counterexamples found | not-found |
| closest-quota | filled | 2 counterexamples found | not-found |
| triad-rite | filled | 2 counterexamples found | not-found |
| flood-line | filled | 2 counterexamples found | not-found |

One curated-input bug found by S10 itself: candle-compact's final hidden
case was missing its second argument; fixed and re-validated.

## Review round

Ten reviewer inputs prepared under `target/pack-review/`; ten summaries
written to `docs/pack-reviews/` and all passed `SUM-01`. The pack secret
scan over the new public trees and summaries is clean.

All ten approved under the delegated publication instruction of
21 November 2026: `status = "published"` in this commit; `approval.toml`
per slug lands in the private repo stamped with this commit's hash.

## Review round 3 (private-repo PIP-12 fix)

Devin review on the private approvals PR found five curated hidden cases
duplicating public samples (flood-line 2, closest-quota 1, harbour-depths
1, sigil-dedup 1, triad-rite 1). Each was replaced with a distinct
schema-valid input whose expected value was re-derived from the pack's
reference; `sbx_batch02` was re-run and passed 45.12s. The five affected
approvals were restamped with canonical manifest hashes computed over git
blob bytes, and the five review summaries now record the fresh assembly
hashes (private PR #8, this commit).

## Import and publish — batch 02 (production)

`counterspell-catalogue import --packs <public> --private-root <private>
--deny-warnings` ran on the prod host inside the server image
(`021e939b`) against the prod rootless sandbox daemon. All ten batch-02
packs reported `created version=1`; every batch-01 and earlier pack
reported `unchanged`; the three rotated starters refused
`version_not_bumped` as before. Each of the ten then went through
`publish` (`created version=1`). Approval hashes matched the importer's
canonical manifest on the first pass — they were computed over git blob
bytes via `scripts/pack-content-hash-git.py`.

## Production verification (B02-06)

- `GET /api/problems` serves 25 problems including all ten batch-02
  slugs with correct tracks, difficulties and concepts.
- `GET /api/problems/sigil-dedup` and `/flood-line` return full
  statements and samples.
- A temporary `ops-verify-b02` account submitted a real `submit` run for
  `sigil-dedup` (Python known-correct fixture): `94c2556f`, accepted
  27/27 in 274ms through the prod rootless sandbox.
- Cleanup: ops session and user row deleted; all staging, env file,
  scripts and logs removed from the host (verified empty).
