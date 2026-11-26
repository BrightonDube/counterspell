# The Deepest Lex

## Task
The lexicon holds word entries. Call `longest_word(words)` and return the length of the longest dictionary entry.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

## Output
The longest entry's length; `0` when the dictionary is empty.

## Examples
- `longest_word("cat,lantern,dog")` returns `7`.
- `longest_word("")` returns `0`.
- `longest_word("a,bb,ccc")` returns `3`.

## Constraints
- `0 <= len(words) <= 40`
- `words` uses `a`-`z` and `,`
