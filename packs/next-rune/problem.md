# The Next Rune

## Task
The lexicon completes a partial word. Call `next_letters(words, prefix)` and return the number of distinct runes that can come right after `prefix` — counting each different next rune once.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

`prefix` is lowercase `a`-`z` and may be empty. An entry equal to `prefix` offers no next rune; an entry with `prefix` in the middle or at the end is not a completion.

## Output
The number of distinct runes that extend `prefix` inside the dictionary; `0` when nothing completes it.

## Examples
- `next_letters("cat,car,cow,dog", "ca")` returns `2` — `t` and `r`.
- `next_letters("cat,car,cow,dog", "d")` returns `1` — only `o`.
- `next_letters("cat,dog", "x")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`, `0 <= len(prefix) <= 12`
- `words` uses `a`-`z` and `,`; `prefix` uses `a`-`z`
