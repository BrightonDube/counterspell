# The Quorum Votes

## Task
A ward council of `n` keepers can only bind a rite when a strict majority votes aye. Call `quorum_reached(n, votes)` and return `1` when the votes bind, `0` when they fall short.

## Input
`n` keepers (`0` to `30`), `votes` ayes cast (`0` to `30`).

## Output
`1` when `votes` exceeds half of `n` (strictly more than `n / 2`), else `0`.

## Examples
- `quorum_reached(5, 3)` returns `1` — three of five binds.
- `quorum_reached(4, 2)` returns `0` — a tied council cannot bind.
- `quorum_reached(0, 0)` returns `0`.

## Constraints
- `0 <= n <= 30`, `0 <= votes <= 30`
