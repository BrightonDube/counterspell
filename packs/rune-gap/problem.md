# The Rune Gap

## Task
The satchel can hold runes whose marks total at most `cap`. Call `gap_count(marks, cap)` and return how many subsets of the pile fit — the empty subset counts.

## Input
`marks` holds the rune marks; `cap` is the satchel's limit.

## Output
The number of subsets whose marks total at most `cap`.

## Examples
- `gap_count([1, 2, 3], 4)` returns `6` — `{}`, `{1}`, `{2}`, `{3}`, `{1,2}` and `{1,3}`.
- `gap_count([], 0)` returns `1`.
- `gap_count([5, 5], 4)` returns `1` — only the empty subset fits.

## Constraints
- `0 <= len(marks) <= 30`, `0 <= marks[i] <= 20`, `0 <= cap <= 60`
