# Pair Count

## Task

Given a list of integers `bones` and an integer `target`, count the pairs of positions `i` and `j` where `i < j` and `bones[i] + bones[j] == target`. Each pair is counted once.

## Input

- `bones`: a list of integers with between 0 and 30 items; each item is between -50 and 50.
- `target`: an integer between -100 and 100.

## Output

An integer: the number of pairs. An empty list has no pairs, so the count is 0.

## Examples

| bones | target | result |
|---|---|---|
| `[1, 2, 3, 4]` | `5` | `2` |
| `[0, 0, 0]` | `0` | `3` |
| `[]` | `0` | `0` |

## Constraints

- `bones` has between 0 and 30 items.
- Each item is an integer between -50 and 50.
- `target` is an integer between -100 and 100.

This is a test pack used only by the ghost engine's conformance tests. It is not in the catalogue.
