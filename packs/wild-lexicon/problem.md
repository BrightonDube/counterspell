# The Wild Lexicon

## Task
A search pattern reaches the lexicon with wild runes inside. Call `count_matches(words, pattern)` and return the number of distinct entries the pattern matches.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

`pattern` uses lowercase `a`-`z` plus the wild rune `*`. A `*` stands for any run of runes — including an empty run. Every other rune matches only itself, in order, from the first rune of the entry to the last.

## Output
The number of distinct entries matched by `pattern`; `0` when nothing matches.

## Examples
- `count_matches("cat,cart,cog", "c*")` returns `3` — the wild rune takes the rest.
- `count_matches("cat,cart,dog", "c*t")` returns `2` — `*` is `a` in `cat` and `ar` in `cart`.
- `count_matches("cat,dog", "d*g")` returns `1`.

## Constraints
- `0 <= len(words) <= 40`, `0 <= len(pattern) <= 12`
- `words` uses `a`-`z` and `,`; `pattern` uses `a`-`z` and `*`
