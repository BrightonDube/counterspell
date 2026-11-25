# The Candle Ledger

## Task
The vault ledger lists the burn size of every candle inspected. Call `top_burn(sizes)` and return the burn size that was logged most often, as an integer. When sizes tie, return the smallest of them. An empty ledger returns `0`.

## Input
A list of burn sizes. Each size is from `1` to `9`.

## Output
Return the most frequently logged burn size as an integer, or `0` for an empty ledger.

## Examples
`top_burn([2, 7, 2, 3, 2, 7])` returns `2`.
Size `2` was logged three times and size `7` twice, so `2` wins.

## Constraints
The list has at most 40 entries. Each entry is from `1` to `9`.
