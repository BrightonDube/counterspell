# The Lock Order

## Task
Each rite seizes wards in a fixed order, written onto one flat tape — `lens[i]` says how many entries on `wards` belong to rite `i`. If one rite holds ward `a` before `b` while another takes `b` before `a`, the pair can seize each other's next ward forever. Call `lock_order(wards, lens)` and return `1` when every shared ward pair keeps one order, else `0`.

## Input
`wards` is the flat tape of all ward orders, laid rite after rite (`0` to `30`); `lens` lists each rite's stretch on the tape (`1` to `8`). The lens sum equals the tape length.

## Output
`1` for a consistent order, `0` for a conflict.

## Examples
- `lock_order([1, 2, 2, 1], [2, 2])` returns `0` — rite 0 takes `1` then `2`, rite 1 takes `2` then `1`.
- `lock_order([1, 2, 3, 1, 3], [3, 2])` returns `1`.
- `lock_order([1, 2, 3, 4], [2, 2])` returns `1` — the rites share no ward pair.

## Constraints
- `1 <= len(lens) <= 8`, each `1 <= lens[i] <= 8`; `len(wards) == sum(lens)` with `0 <= ward <= 30`
