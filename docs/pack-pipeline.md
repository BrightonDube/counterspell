# Pack pipeline: the operator's guide

How a topic becomes a Counterspell problem pack, and which parts are yours.

## The shape of a pack

A pack lives in two places. The **public tree** (`packs/<slug>/`) holds the
statement, samples, hints, and metadata - everything a solver may see. The
**private tree** (the separate private repo) holds hidden tests, the reference
solution, the deliberately-wrong "ghost" programs, and your approval file. The
validator checks them together; nothing secret ever ships to solvers.

## What runs by itself

1. An agent drafts the pack through stages S0-S9: statement, samples, hidden
   cases, generator, ghosts, then a static validation pass.
2. `scripts/pack-batch.sh` can run several slugs through those static stages
   headless. It stops at `static_complete`, never runs code, never publishes,
   and never writes `approval.toml`.
3. A reviewer round scores the pack; at most three rounds run before the pack
   is marked `escalated` for you.

## What only you run

- **S10/S11 sandbox checks** - on the host, with your approval for that
  session's code-running tools. The judge's rootless sandbox executes the
  reference and each ghost; results are recorded with the image digest.
- **The approval gate (S13)** - read `docs/pack-reviews/<slug>.md` and the
  private report. To approve: set `status = "published"` in the approval
  commit and write `approval.toml` in the private repo with the content hash.
- **The import** - you run `counterspell-catalogue import --sandbox
  --private-root <checkout>`; the importer refuses changed files (APR-01).

## Reading the summary

`docs/kiro-log/pack-batch-<date>.md` lists codes and counts per slug - never
hidden values. A slug reaching `awaiting_approval` is ready for your review.

## What `draft` means

Every new pack is `draft`. Draft packs validate and can be worked on, but the
API does not serve them to solvers. Only your approval commit changes that.

## How to reject

Move the state file back to `rework` with a note, or just close the pull
request - the next round re-enters the review loop with a fresh content hash.

## Rotation

When hidden values may have leaked (for example, they once sat in public git
history), `.kiro/pipeline/rotation.md` describes the rotation: record retired
values as SHA-256 hashes in the private `retired-values.toml`, write new
hidden data, bump `pack_version`, and re-run the stages. You approve any
rotation before it runs.
