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

## Catalogue batch 02 — two pointers

Ten original ideas for batch 02 (completion-program CP16): four easy, four
medium, two hard. Every concept below exists in `packs/catalogue.toml`
(TAX-01), and no slug is already used by a pack (SLUG-01). The theme is
opposing scans, partitioning, deduplication and pair or triplet
relationships; supported call types stay `int`, `str`, `list[int]` and
`list[list[int]]`.

| Slug | Track | Concepts | Difficulty | Original idea | Planned ghost kinds |
|---|---|---|---|---|---|
| sigil-dedup | arrays | arrays, two-pointers, deduplication | easy | A sorted tally of sigil marks holds repeats; report how many distinct marks it carries. | counting-all-entries, off-by-one-write-index |
| ember-drift | arrays | arrays, two-pointers, partitioning | easy | Drift every dark lantern to the far end while the lit order stays. | unstable-partition, overwriting-unread |
| twin-tides | arrays | arrays, two-pointers, merging | easy | Merge two sorted tide tables into one ordered table. | stopping-at-first-end, comparing-wrong-heads |
| candle-compact | arrays | arrays, two-pointers, edge-cases | easy | Remove every entry equal to the cursed sigil; report how many candles remain. | keeping-count-not-compacting, index-drift |
| harbour-depths | arrays | arrays, two-pointers, pairs | medium | Two walls and a gap: find the deepest water the harbour line can hold between any two marks. | shrinking-both-ends, wrong-area-formula |
| sorted-embers | arrays | arrays, two-pointers, edge-cases | medium | Ember readings arrive sorted with negatives; return their squares sorted. | squaring-in-place-order, appending-backwards |
| shared-sigils | arrays | arrays, two-pointers, merging | medium | Two sorted sigil lists; report the markings they share, in order, without repeats. | advancing-both-on-mismatch, repeating-shared-mark |
| closest-quota | arrays | arrays, two-pointers, pairs | medium | Sorted marks and a quota; find the pair sum nearest the quota. | stopping-at-first-pair, signed-vs-absolute-gap |
| triad-rite | arrays | arrays, two-pointers, nested-loops | hard | Count the triads in a sorted tally whose three marks sum to the quota exactly. | counting-permutations, skipping-duplicate-triads |
| flood-line | arrays | arrays, two-pointers, prefixes | hard | After the storm, measure how much rain the wall line traps between its posts. | min-vs-max-walls, recomputing-every-gap |

Balance check: 10 arrays track; 4 easy, 4 medium, 2 hard. The batch asks for
opposing scans (twin-tides, sorted-embers, harbour-depths, flood-line),
partitioning and deduplication (ember-drift, candle-compact, sigil-dedup,
shared-sigils) and pair or triplet relationships (closest-quota,
harbour-depths, triad-rite). Quadratic ghosts tempt a rescan in
harbour-depths, flood-line and triad-rite.

## Catalogue batch 03 — stacks, queues and expression processing

Ten original ideas for batch 03 (completion-program CP16): four easy, four
medium, two hard. New concepts stacks / queues / simulation / parsing were
added to `packs/catalogue.toml` (TAX-01); no slug is already used by a pack
(SLUG-01). Supported call types stay `int`, `str`, `list[int]` and
`list[list[int]]`. Every semantic is defined for arbitrary in-schema input,
so generated hidden cases are always meaningful.

| Slug | Track | Concepts | Difficulty | Original idea | Planned ghost kinds |
|---|---|---|---|---|---|
| rune-brackets | strings | strings, stacks, edge-cases | easy | An inscription of `()[]{}` runes; report `1` when every bracket nests correctly, else `0`. | checking-only-parens, empty-is-invalid |
| stack-sigils | arrays | arrays, stacks, simulation | easy | Marks above `0` are laid on the sigil stack and each `0` lifts one; return the top mark (`-1` when bare). | pop-before-push-order, missing-empty-check |
| cauldron-queue | arrays | arrays, queues, simulation | easy | Marks above `0` join the cauldron line and each `0` serves the front; return how many were served (serving an empty line does nothing). | serving-the-back, counting-joins |
| deepest-ember | strings | strings, stacks, scanning, edge-cases | easy | Scan an `()` inscription and report the deepest correctly-opened nesting ever held; a stray closer is ignored. | counting-closers-not-depth, resetting-on-stray |
| undo-ritual | strings | strings, stacks, simulation | medium | Each `#` in the chant erases the previous surviving rune; return the inscription that remains. | erasing-the-wrong-side, index-rewind |
| serving-order | arrays | arrays, queues, simulation, edge-cases | medium | Same line rules as cauldron-queue; return the marks served, in order, as a list. | returning-waiters-not-served, serve-before-join |
| min-add | strings | strings, stacks, counting | medium | Report the fewest `(` or `)` insertions that would balance an `()` inscription. | balancing-length-not-structure, ignoring-surplus-openers |
| next-lantern | arrays | arrays, stacks, scanning | medium | For each lantern, report the steps until the next strictly taller lantern; `0` when none follows. | first-smaller-not-taller, distance-to-end |
| wall-span | arrays | arrays, stacks, edge-cases | hard | A wall line of heights; return the largest rectangular stone slab that fits inside one unbroken stretch of wall (largest rectangle under the silhouette). | min-vs-own-height, skipping-end-boundary |
| longest-rite | strings | strings, stacks, parsing, edge-cases | hard | Report the length of the longest stretch of the `()` inscription that is correctly nested. | valid-prefix-only, missing-index-anchor |

Balance check: 5 strings track, 5 arrays track; 4 easy, 4 medium, 2 hard.
Covers nesting recognition (rune-brackets, ember-depth), stack and queue
simulation (stack-sigils, cauldron-queue, undo-ritual, serving-order),
monotonic structure (next-lantern, wall-span) and bounded repair or
longest-valid analysis (min-add, longest-rite). Ghosts tempt the wrong
reservation order, missing empty-structure checks and depth-vs-count
confusion.

## Catalogue batch 04 — sliding windows and streaming statistics

Ten original ideas for batch 04 (completion-program CP16): four easy, four
medium, two hard. New concepts sliding-window / streaming were added to
`packs/catalogue.toml` (TAX-01); no slug is already used by a pack
(SLUG-01). Supported call types stay `int`, `str`, `list[int]` and
`list[list[int]]`. Every semantic is defined for arbitrary in-schema
input, so generated hidden cases are always meaningful.

| Slug | Track | Concepts | Difficulty | Original idea | Planned ghost kinds |
|---|---|---|---|---|---|
| warm-window | arrays | arrays, sliding-window, scanning | easy | Largest sum over any `width` consecutive readings (`0` when the log is shorter). | first-window-only, dropping-last-window, max-reading |
| quiet-hours | arrays | arrays, sliding-window, counting, edge-cases | easy | Count three-hour stretches whose readings add to no more than the quota. | strict-vs-inclusive-quota, partial-edge-windows, wrong-width |
| bead-run | strings | strings, scanning, counting, edge-cases | easy | Longest run of one repeated bead on an `a`/`b` strand. | most-frequent-not-run, final-run-only, counting-groups |
| calm-spell | strings | strings, scanning, edge-cases, counting | easy | Longest stretch of an `a`/`b`/`#` chant with no `#` inside. | total-non-crack-count, first-spell-only, last-spell-only |
| min-varied | strings | strings, sliding-window, frequency-maps | medium | Shortest fragment of an `a`-`d` frieze holding at least one `a`, `b` and `c` (`0` when none). | length-three-only, distinct-count, longest-window |
| thrifty-pane | strings | strings, sliding-window, frequency-maps | medium | Longest pane of an `a`/`b`/`c` strip showing at most two rune kinds. | at-most-three, same-rune-runs, all-or-nothing |
| drift-alarm | arrays | arrays, sliding-window, streaming | medium | Count hours whose reading tops every reading in the previous `width` hours. | self-inclusive-window, non-strict, any-not-all |
| harvest-quota | arrays | arrays, sliding-window, prefixes, edge-cases | medium | Longest run of vine marks summing to no more than the quota. | strict-quota, prefix-only, counting-windows |
| gale-peaks | arrays | arrays, sliding-window, queues | hard | The peak reading of every `width`-hour gust window, in order. | first-reading-as-peak, stale-peak, last-window-only |
| three-rune-windows | strings | strings, sliding-window, counting | hard | Count contiguous panes of an `a`/`b`/`c` band showing all three runes. | fixed-length-three, presence-only, off-by-one-tail |

Balance check: 5 strings track, 5 arrays track; 4 easy, 4 medium, 2 hard.
Covers fixed-width aggregation (warm-window, quiet-hours), run and
segment scanning (bead-run, calm-spell), variable windows over frequency
maps (min-varied, thrifty-pane), look-back and sum-bounded windows
(drift-alarm, harvest-quota) and the monotonic-deque peak plus
count-the-tail arguments (gale-peaks, three-rune-windows). Ghosts tempt
boundary off-by-ones, wrong-width slides and count-vs-measure confusion.

## Batch 05 — binary search and ordered decision spaces

Ten packs on exact lookup, boundaries, rotated order and answer-space search.

| Slug | Title | Difficulty | Track |
| ---- | ----- | ---------- | ----- |
| ember-index | The Ember Index | easy | arrays |
| first-clear | First Clear Morning | easy | arrays |
| count-below | Count Below | easy | arrays |
| highest-floor | The Highest Floor | easy | arrays |
| ember-edges | Ember Edges | medium | arrays |
| peak-torch | Peak of the Torches | medium | arrays |
| rotated-pantry | The Rotated Pantry | medium | arrays |
| brew-pace | The Brew Pace | medium | arrays |
| valley-low | The Valley Low | hard | arrays |
| even-offering | The Even Offering | hard | arrays |
