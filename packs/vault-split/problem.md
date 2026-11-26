# The Vault Split

## Task
A vault of marks is split between two keepers. Call `split_diff(marks)` and return the smallest difference the two shares can hold.

## Input
`marks` lists the worths; each keeper takes some of them.

## Output
The smallest difference between the shares.

## Examples
- `split_diff([1, 2, 7])` returns `4` — `1+2` against `7`.
- `split_diff([1, 5, 11, 5])` returns `0` — `1+5+5` against `11`.
- `split_diff([7, 3])` returns `4`.

## Constraints
- `0 <= len(marks) <= 24`; each worth is `1..100`
