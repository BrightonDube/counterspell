# The Twin Stems

## Task
The lexicon holds word entries. Call `prefix_pairs(words)` and return the number of unordered pairs of distinct entries where one is a proper prefix of the other.

## Input
`words` holds the dictionary as one string: word entries separated by commas, all lowercase `a`-`z`. Empty entries (a comma next to a comma, or a comma at an end) are not words and are ignored. Entries may repeat and each repetition counts on its own.

A proper prefix is strictly shorter than the whole entry — identical entries never pair. Pairs are unordered: `cat` with `cart` is one pair, not two.

## Output
The number of distinct-entry pairs where one is a proper prefix of the other; `0` when no pair exists.

## Examples
- `prefix_pairs("car,cart,dog")` returns `1` — `car` leads `cart`.
- `prefix_pairs("a,ab,abc")` returns `3` — `a`-`ab`, `a`-`abc`, `ab`-`abc`.
- `prefix_pairs("cat,dog")` returns `0`.

## Constraints
- `0 <= len(words) <= 40`
- `words` uses `a`-`z` and `,`
