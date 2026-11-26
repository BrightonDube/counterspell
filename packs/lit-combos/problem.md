# The Lit Combos

## Task
A toll may be paid in kinds of `coins`, each reusable. Call `count_mixes(coins, target)` and return how many distinct mixes pay exactly — mixes differing only in order count once.

## Input
`coins` lists the kinds; `target` is the toll. Every kind is `1..60`.

## Output
The mix count; a zero toll has one empty mix, and a hopeless toll has none.

## Examples
- `count_mixes([1, 2, 5], 5)` returns `4` — `5`, `2+2+1`, `2+1+1+1` and `1×5`.
- `count_mixes([2], 3)` returns `0`.
- `count_mixes([1], 0)` returns `1`.

## Constraints
- `1 <= len(coins) <= 12` kinds, each `1..60`; `0 <= target <= 300`
