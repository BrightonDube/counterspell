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
