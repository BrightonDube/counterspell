# The Cover Pick

## Task
A ward of `n` towers must be lit by talismans — each talisman mask is a bitset over the towers. The rite is greedy: each turn it takes the talisman covering the most still-dark towers (ties to the lowest index) until every tower burns or nothing adds light. Call `cover_pick(n, masks)` and return how many talismans the rite burns.

## Input
`n` the tower count (`0` to `20`), `masks` the talisman bitsets — bit `t` of `masks[i]` lights tower `t`. Bits outside `0..n-1` name no tower and are ignored.

## Output
The number of picks the greedy rite makes before the ward is lit or no mask adds new light.

## Examples
- `cover_pick(3, [3, 6, 1])` returns `2` — mask `0b011` covers two towers, then `0b110` lights the last.
- `cover_pick(4, [15])` returns `1` — one talisman lights all four.
- `cover_pick(3, [0, 0])` returns `0` — dark talismans light nothing.

## Constraints
- `0 <= n <= 20`, `0 <= len(masks) <= 24`, `0 <= masks[i] <= 2^20 - 1`
