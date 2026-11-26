# The Lexicon Shrink

## Task
The lexicon holds word entries; any entry that is a proper prefix of another entry gets absorbed. Call `shrunk_count(words)` and return how many distinct entries survive the shrink.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

Survivors are counted over distinct words — repeated entries name the same word.

## Output
The number of distinct entries that are not a proper prefix of any other entry; `0` when the dictionary is empty.

## Examples
- `shrunk_count("car,cart,dog")` returns `2` — `car` is absorbed into `cart`.
- `shrunk_count("a,ab,abc")` returns `1` — only `abc` survives.
- `shrunk_count("")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`
- `words` uses `a`-`z` and `,`
