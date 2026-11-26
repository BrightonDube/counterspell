# The Mod Toll

## Task
A ferryman charges the toll's remainder — the sum of the pile, taken modulo `m`. Negative coins count against you, and the toll is never negative. Call `mod_toll(values, m)` and return the toll.

## Input
`values` lists integers from `-20` to `20`; `m` runs `1` to `20`.

## Output
`sum(values) mod m` as a value in `0..m-1`, even when the sum is negative.

## Examples
- `mod_toll([7, 8, 9], 5)` returns `4` — `24 mod 5`.
- `mod_toll([-1, 2], 5)` returns `1`.
- `mod_toll([], 7)` returns `0`.

## Constraints
- `0 <= len(values) <= 30`, each `-20..20`; `1 <= m <= 20`
