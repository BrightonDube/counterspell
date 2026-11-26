# The Median Ward

## Task
The ward holds marks; its pivot is the median. Call `median_x2(marks)` and return twice the median so the answer stays a whole number: twice the middle mark on odd counts, the sum of the two middle marks on even counts.

## Input
`marks` holds the marks. An empty ward answers `0`.

## Output
Twice the median: `2 * middle` on odd counts, `lower + upper` of the two middles on even counts.

## Examples
- `median_x2([3, 1, 2])` returns `4` — the middle is `2`.
- `median_x2([4, 1, 3, 2])` returns `5` — the middles `2` and `3` sum to `5`.
- `median_x2([])` returns `0`.

## Constraints
- `0 <= len(marks) <= 40`, `-999 <= marks[i] <= 999`
