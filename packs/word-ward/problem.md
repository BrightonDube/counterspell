# The Word Ward

## Task
The lexicon holds word entries. Call `has_word(words, word)` and return `1` when `word` is a complete dictionary entry and `0` when it is missing — including when it is only a prefix of real entries.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

`word` is lowercase `a`-`z` and may be empty; the empty string is never a word.

## Output
`1` when `word` matches an entry exactly, `0` otherwise.

## Examples
- `has_word("cat,car,dog", "car")` returns `1`.
- `has_word("cat,car,dog", "ca")` returns `0` — `ca` is only a prefix.
- `has_word("", "cat")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`, `0 <= len(word) <= 12`
- `words` uses `a`-`z` and `,`; `word` uses `a`-`z`
