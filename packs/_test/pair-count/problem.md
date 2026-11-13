# Pair Count

Given a list of whole numbers `bones` and a whole number `target`, count the pairs of positions `i` and `j` where `i < j` and `bones[i] + bones[j] == target`.

- Each pair is counted once. The order of the two positions does not matter because `i < j` is required.
- An empty list has no pairs, so the count is 0.

## Function

Write `pair_count(bones, target)`. Return the number of pairs as an integer.

## Constraints

- `bones` has between 0 and 30 items.
- Each item is between -50 and 50.
- `target` is between -100 and 100.

## Examples

| bones | target | result |
|---|---|---|
| `[1, 2, 3, 4]` | `5` | `2` |
| `[0, 0, 0]` | `0` | `3` |
| `[]` | `0` | `0` |

This is a test pack used only by the ghost engine's conformance tests. It is not in the catalogue.
