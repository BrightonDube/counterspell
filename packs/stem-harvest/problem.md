# The Stem Harvest

## Task
Every word in the lexicon sits at a depth: the length of the longest distinct word that is a proper prefix of it, or `0` when no word leads it. Call `stem_depth_sum(words)` and return the total depth over the distinct entries.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

A proper prefix is strictly shorter than the word itself; a word never leads itself.

## Output
The sum of depths across the distinct entries; `0` for an empty dictionary.

## Examples
- `stem_depth_sum("car,cart,dog")` returns `3` — `cart` hangs from `car` at depth 3.
- `stem_depth_sum("a,ab,abc")` returns `3` — depths 0, 1 and 2.
- `stem_depth_sum("cat,dog")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`
- `words` uses `a`-`z` and `,`
