# The Closest Quota

## Task
A rite needs a quota, and a sorted column of ember marks is offered in pairs: any two distinct marks may be weighed together. Call `nearest_pair(marks, quota)` and return the sum of two marks nearest the quota, as an integer. When two sums sit equally near, return the smaller sum. A column with fewer than two marks returns `0`.

## Input
A list of marks ordered smallest first, then the quota the rite needs.

## Output
Return the sum of two marks nearest the quota, as an integer; ties return the smaller sum.

## Examples
`nearest_pair([1, 3, 5, 8], 10)` returns `9`. The pairs sum to `4`, `6`, `9`, `8`, `11` and `13`; `9` and `11` sit equally near the quota, and the tie returns the smaller.

## Constraints
The column has at most 40 marks, ordered smallest first. Each mark is from `0` to `50`, and the quota is from `0` to `100`.
