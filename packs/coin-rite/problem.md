# The Coin Rite

## Task
A rite pays `target` in coin kinds of `coins` — as many of each kind as needed. Call `coin_rite(coins, target)` and return the fewest coins that pay exactly.

## Input
`coins` lists the kinds; `target` is the toll. Every kind is `1..60`.

## Output
The fewest coins, or `-1` when no mix pays the toll.

## Examples
- `coin_rite([1, 2, 5], 11)` returns `3` — `5 + 5 + 1`.
- `coin_rite([2], 3)` returns `-1`.
- `coin_rite([1], 0)` returns `0` — the toll is already paid.

## Constraints
- `1 <= len(coins) <= 12` kinds, each `1..60`; `0 <= target <= 300`
