# The Lantern Streak

## Task
A row of lanterns ends a street. Each lantern is lit or dark. Call `lit_streak(lights)` and return the number of lanterns lit in a row at the end of the street, as an integer. The count starts at the last lantern and stops at the first dark one.

## Input
A list of marks. Each mark is `1` when the lantern is lit and `0` when it is dark. An empty street has a streak of `0`.

## Output
Return the length of the lit run at the end of the list as an integer.

## Examples
`lit_streak([1, 0, 1, 1, 1])` returns `3`.
The last three lanterns are lit in a row. The dark lantern before them stops the count.

## Constraints
The list has at most 40 marks. Each mark is `0` or `1`.
