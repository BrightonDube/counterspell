# The Rune Dial

## Task
Marks land on the dial one by one. Call `run_median(marks)` and return, after each mark, twice the median so far — twice the middle on odd counts, the two middles' sum on even counts.

## Input
`marks` holds the marks in arrival order. An empty dial answers `[]`.

## Output
A list with one entry per mark: `2 * middle` while the count is odd, `lower + upper` while it is even.

## Examples
- `run_median([1, 3, 2])` returns `[2, 4, 4]`.
- `run_median([5, 5])` returns `[10, 10]`.
- `run_median([])` returns `[]`.

## Constraints
- `0 <= len(marks) <= 40`, `-999 <= marks[i] <= 999`
