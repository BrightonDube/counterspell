# The Sigil Tally

## Task
The warden tallies sigil marks in a long column, always smallest first, so equal marks sit side by side. Call `distinct_sigils(marks)` and return how many different marks the column holds, as an integer. An empty column holds none.

## Input
A list of marks. Marks arrive sorted, smallest first, and equal marks sit together.

## Output
Return the number of different marks as an integer.

## Examples
`distinct_sigils([1, 1, 2, 3, 3])` returns `3`. The column carries the marks `1`, `2` and `3`.

## Constraints
The list has at most 40 marks, ordered smallest first. Each mark is from `0` to `99`.
