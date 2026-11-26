# The Miss Count

## Task
A warded cache holds `c` slots, each bound to one line: a mark lands in slot `mark % c` and stays only while its tag `mark // c` matches. Call `miss_count(marks, c)` and return how many marks arrive to find their slot cold or foreign.

## Input
`marks` lists marks in arrival order (`0` to `60`); `c` is the slot count (`1` to `8`).

## Output
The miss count — arrivals whose slot holds a different tag (or nothing).

## Examples
- `miss_count([0, 1, 0, 4, 0], 2)` returns `4` — mark `4` evicts tag `0` from slot `0`, so the last `0` misses.
- `miss_count([], 4)` returns `0`.
- `miss_count([0, 0, 0], 1)` returns `1`.

## Constraints
- `0 <= len(marks) <= 24` with `0 <= mark <= 60`, `1 <= c <= 8`
