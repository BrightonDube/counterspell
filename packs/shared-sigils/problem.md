# The Shared Sigils

## Task
Two grimoires each keep a column of sigil marks, smallest first. Call `shared_marks(first, second)` and return the marks both grimoires carry, in order, each listed once. A mark that repeats inside one grimoire is shared only once. Two empty columns return an empty list.

## Input
Two lists of sigil marks, each ordered smallest first.

## Output
Return the marks present in both lists, ordered smallest first, with each mark listed once.

## Examples
`shared_marks([1, 1, 2, 4], [1, 2, 2, 3])` returns `[1, 2]`. Both grimoires carry `1` and `2`; the `4` and `3` belong to only one.

## Constraints
Each column has at most 20 marks, ordered smallest first. Each mark is from `0` to `99`.
