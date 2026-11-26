# The Split Ward

## Task
The lexicon holds word entries and `s` is a rune string with no separators. Call `can_segment(words, s)` and return `1` when `s` can be cut fully into dictionary entries and `0` when some rune run is left over.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

`s` is lowercase `a`-`z`; the empty string segments trivially into no entries. Entries may be reused any number of times.

## Output
`1` when `s` segments fully into dictionary entries, `0` otherwise.

## Examples
- `can_segment("cat,dog", "catdog")` returns `1`.
- `can_segment("cat,dog", "catbird")` returns `0` — `bird` is missing.
- `can_segment("cat,dog", "")` returns `1` — nothing to cut.

## Constraints
- `0 <= len(words) <= 40`, `0 <= len(s) <= 24`
- `words` uses `a`-`z` and `,`; `s` uses `a`-`z`
