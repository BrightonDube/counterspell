# The Summon Rite

## Task
The keeper asks whether any choice of embers — any subset of the pile, including none — sums to the rite's mark. Call `can_summon(marks, target)` and return `1` when some subset does, `0` otherwise.

## Input
`marks` holds the marks; `target` is the rite's mark. The empty choice sums to `0`.

## Output
`1` when some subset sums to `target`, else `0`.

## Examples
- `can_summon([1, 2, 3], 3)` returns `1` — `{3}` or `{1, 2}`.
- `can_summon([2, 4, 6], 5)` returns `0`.
- `can_summon([], 0)` returns `1` — the empty choice.

## Constraints
- `0 <= len(marks) <= 30`, `-20 <= marks[i] <= 20`, `-40 <= target <= 40`
