# Lantern Streaks

## Task

Write `longest_lit(lanterns)` to count how many lanterns in a row are lit at
the end of the street.

## Input

- `lanterns`: a list of `0` (dark) and `1` (lit) integers, ordered from the
  first house to the last.

## Output

An integer: the length of the lit streak touching the last house.

## Examples

- `longest_lit([1, 0, 1, 1])` returns `2`
- `longest_lit([0, 0, 0])` returns `0`
- `longest_lit([])` returns `0`

## Constraints

- The list may be empty.
- Every value is `0` or `1`.
