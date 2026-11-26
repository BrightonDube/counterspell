# The Prefix Tally

## Task
The lexicon holds word entries. Call `prefix_count(words, prefix)` and return how many entries begin with `prefix`.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

`prefix` is lowercase `a`-`z` and may be empty; every entry begins with an empty prefix.

## Output
The number of dictionary entries starting with `prefix`; a prefix-only entry still counts — the entry only needs to begin with it.

## Examples
- `prefix_count("cat,car,dog", "ca")` returns `2`.
- `prefix_count("cat,car,dog", "x")` returns `0`.
- `prefix_count("", "ca")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`, `0 <= len(prefix) <= 12`
- `words` uses `a`-`z` and `,`; `prefix` uses `a`-`z`
