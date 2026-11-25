# The Triad Rite

## Task
A rite needs three candles whose marks sum to a quota. The candles stand in a column, ordered smallest first, and each choice of three different candles counts as its own triad — even when the marks read the same. Call `triads_at_quota(marks, quota)` and return the count of triads, as an integer. A column with fewer than three candles has none.

## Input
A list of marks ordered smallest first, then the quota the rite needs.

## Output
Return the count of index-distinct triads whose three marks sum to the quota, as an integer.

## Examples
`triads_at_quota([2, 2, 2, 2], 6)` returns `4`. Any three of the four candles sum to `6`, and four choose three is four.

## Constraints
The column has at most 40 marks, ordered smallest first. Each mark is from `0` to `30`, and the quota is from `0` to `90`.
