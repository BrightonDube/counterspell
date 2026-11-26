# The Leak Check

## Task
A split rite must keep the train scroll and the trial scroll disjoint — any rune id appearing in both is a leak. Call `leak_check(train, test)` and return how many distinct ids leak across.

## Input
`train` and `test` are id lists; duplicates inside one list still count once.

## Output
The count of distinct ids present in both lists.

## Examples
- `leak_check([1, 2, 3], [3, 4])` returns `1`.
- `leak_check([1, 2], [3, 4])` returns `0`.
- `leak_check([], [1])` returns `0`.

## Constraints
- `0 <= len(train), len(test) <= 24`, `0 <= ids <= 999`
