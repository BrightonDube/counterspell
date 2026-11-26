# The Ward Rows

## Task
A ward row of `length` slots is laid with pieces of length 1, 2 or 3, placed in order. Call `row_ways(length)` and return how many distinct piece orders fill the row.

## Input
`length` is the slot count, `0..30`.

## Output
The number of piece orders; an empty row counts as one finished layout.

## Examples
- `row_ways(0)` returns `1`.
- `row_ways(3)` returns `4` — `111`, `12`, `21` and `3`.
- `row_ways(4)` returns `7`.

## Constraints
- `0 <= length <= 30`
