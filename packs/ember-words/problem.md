# The Ember Words

## Task
A scrawled line must break into lexicon embers. Call `word_break(text, lexicon)` and return `1` when the whole line splits into comma-listed words, reused freely, or `0` when it cannot.

## Input
`text` is the scrawl; `lexicon` lists the embers comma-joined, no spaces.

## Output
`1` when a full splitting exists, else `0`.

## Examples
- `word_break("candlelight", "candle,light")` returns `1`.
- `word_break("catsandog", "cats,dog,sand,and,cat")` returns `0` — the s is trapped between splits.
- `word_break("aab", "a,ab")` returns `1`.

## Constraints
- `0 <= len(text) <= 60`; `0 <= len(lexicon) <= 200` characters of embers and commas
