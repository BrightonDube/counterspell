# Sigil Groups

## Task
Every rune marking carries a sigil: the sum of its decimal digits. The sigil of `47` is `4 + 7`, which is `11`. Call `largest_sigil(markings)` and return the size of the largest group of markings that share one sigil, as an integer.

## Input
A list of markings. Each marking is from `0` to `999`.

## Output
Return the largest group size as an integer, or `0` for an empty list.

## Examples
`largest_sigil([47, 83, 29, 11])` returns `3`.
The sigils are `11`, `11`, `11` and `2`. Three markings share the sigil `11`.

## Constraints
The list has at most 40 markings. Each marking is from `0` to `999`.
