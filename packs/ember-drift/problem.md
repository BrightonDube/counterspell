# The Ember Drift

## Task
A shelf of ember marks carries `0` where an ember has gone dark. Call `drift_dark(marks)` and return the marks rearranged so every dark ember drifts to the end while the lit embers keep their order. An empty shelf returns an empty list.

## Input
A list of ember marks. `0` marks a dark ember; any other mark is lit.

## Output
Return the rearranged list: every lit mark first, in their original order, then every dark mark.

## Examples
`drift_dark([1, 0, 2, 0, 3])` returns `[1, 2, 3, 0, 0]`. The lit embers `1`, `2` and `3` keep their order; both dark embers drift to the end.

## Constraints
The list has at most 40 marks. Each mark is from `0` to `9`.
