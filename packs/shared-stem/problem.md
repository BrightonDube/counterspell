# The Shared Stem

## Task
The lexicon holds word entries. Call `common_stem(words)` and return the length of the longest run of leading runes every entry shares.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

## Output
The length of the shared leading rune run; `0` when the dictionary is empty or the entries share nothing.

## Examples
- `common_stem("cart,car,cargo")` returns `3` — every entry begins `car`.
- `common_stem("cat,dog")` returns `0`.
- `common_stem("")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`
- `words` uses `a`-`z` and `,`
