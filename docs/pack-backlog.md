# Pack authoring backlog (PA21)

Seven original ideas for the next authoring rounds, balanced across the three
tracks and including medium problems. `lantern-streaks` is the dry-run topic.
Every concept below already exists in `packs/catalogue.toml` (TAX-01), and no
slug is already used by a pack (SLUG-01). Ideas are original one-liners - they
name no third-party problem and describe no third-party text (PIP-3, PIP-4).

| Slug | Track | Concepts | Difficulty | Original idea | Planned ghost kinds |
|---|---|---|---|---|---|
| lantern-streaks | arrays | arrays, edge-cases, scanning | easy | Count how many lanterns in a row are lit at the end of the street. | wrong-initial-value, off-by-one |
| tide-table | arrays | arrays, scanning, edge-cases | easy | Given hourly tide heights, report the first hour the water drops after a peak. | strict-vs-loose-comparison, missing-boundary |
| shelf-balance | arrays | arrays, pairs, indexing | medium | Pair books from both ends of a shelf until the heaviest pair is found; return its total weight. | same-index-pairing, unordered-pair |
| mosaic-row | matrices | matrices, indexing, edge-cases | medium | A mosaic row reads its mirrored tile from the far edge when a tile is chipped; total the visible pigment. | wrong-mirror-index, row-col-confusion |
| ferry-log | strings | strings, scanning, edge-cases | easy | From a comma-separated crossing log, count crossings that ended on the far bank. | case-sensitivity, trailing-delimiter |
| signpost-chain | strings | strings, scanning | medium | Each signpost names the next village; follow the chain until it loops and report the loop's length. | visited-set-missed, off-by-one-count |
| rain-gauges | matrices | matrices, nested-loops, edge-cases | medium | Given a grid of rain gauges, find the driest 2x2 field and report its total. | wrong-start-corner, inclusive-exclusive-bound |

Balance check: 3 arrays, 2 strings, 2 matrices; 3 easy, 4 medium.

## Catalogue batch 01 — arrays and hashing

Ten original ideas for batch 01 (completion-program CP16): four easy, four
medium, two hard. Every concept below exists in `packs/catalogue.toml`
(TAX-01), and no slug is already used by a pack (SLUG-01). Ideas are original
one-liners — they name no third-party problem and describe no third-party
text (PIP-3, PIP-4). `lantern-streaks` and `tide-table` carry over from the
launch-round backlog above; the rest are new.

| Slug | Track | Concepts | Difficulty | Original idea | Planned ghost kinds |
|---|---|---|---|---|---|
| lantern-streaks | arrays | arrays, scanning, edge-cases | easy | Count how many lanterns in a row are lit at the end of the street. | wrong-initial-value, off-by-one |
| tide-table | arrays | arrays, scanning, edge-cases | easy | Given hourly tide heights, report the first hour the water drops after a rise. | strict-vs-loose-comparison, missing-boundary |
| candle-frequency | arrays | arrays, frequency-maps, scanning | easy | Burn-size tallies in the vault: report the size that was logged most often, smallest on a tie. | returning-frequency-not-value, tie-order |
| twin-echoes | arrays | arrays, frequency-maps, edge-cases | easy | Count how many distinct echo markings repeat at least once. | counting-occurrences-not-distinct, threshold-off-by-one |
| sigil-groups | arrays | arrays, grouping, frequency-maps | medium | Group rune markings by the sum of their digit sigils; report the largest group. | grouping-by-value, digit-sum-of-index |
| prefix-echoes | arrays | arrays, prefixes, indexing | medium | Answer each gatekeeper's question: how many times a mark appears among the first k entries. | inclusive-exclusive-bound, whole-list-rescan |
| twin-ledgers | arrays | arrays, hashing, grouping | medium | Join two vault ledgers by key; count keys whose paired totals match. | join-by-position, lost-duplicate-key |
| harvest-tally | arrays | arrays, pairs, nested-loops | medium | Count cross-row sack pairs; the tally can exceed a small counter. | ordered-pair-double-count, same-row-pairs |
| warden-sums | arrays | arrays, prefixes, frequency-maps | hard | Count the reading spans whose net change matches the ritual quota. | start-at-zero-only, stop-at-first-hit |
| equal-embers | arrays | arrays, prefixes, hashing | hard | Find the longest stretch where coals and sparks appear equally often. | recount-every-window, missed-first-seen |

Balance check: 10 arrays track; 4 easy, 4 medium, 2 hard. Batch design asks
for counting, duplicate detection, grouped signatures, prefix-frequency
queries, sparse indexing and multi-key joins — covered by candle-frequency /
twin-echoes, sigil-groups, prefix-echoes, harvest-tally and twin-ledgers —
plus a prefix-information hash-indexed hard pack (equal-embers), tempting
quadratic ghosts (harvest-tally, warden-sums, equal-embers, prefix-echoes)
and large-count handling (harvest-tally, warden-sums).
