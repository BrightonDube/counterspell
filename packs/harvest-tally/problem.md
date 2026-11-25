# The Harvest Tally

## Task
Granary rows hold sacks, and `rows` lists the sack count of each row. The feast pairs every sack with every sack from a different row. Call `cross_pairs(rows)` and return the number of distinct cross-row sack pairs, as an integer. The tally may be very large. A granary with at most one row returns `0`.

## Input
A list of sack counts, one per row.

## Output
Return the number of distinct sack pairs drawn from different rows, as an integer.

## Examples
`cross_pairs([2, 3])` returns `6`.
Each of the `2` sacks in the first row pairs with each of the `3` in the second.

## Constraints
The list has at most 200 rows. Each count is from `0` to `500`.
