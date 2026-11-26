# The Gap Blocks

## Task
Wax seals that touch or overlap run into one block. Call `block_count(spans)` and return how many separate waxed blocks the seals form — a seal at the running end joins the block.

## Input
`spans` lists the `[a, b]` seals, `0 <= a <= b <= 100`.

## Output
The number of separate blocks; a bare lintel has `0`.

## Examples
- `block_count([[0, 1], [2, 3]])` returns `2` — a bare mark splits them.
- `block_count([[0, 3], [4, 5], [2, 4]])` returns `1` — all three touch into one.
- `block_count([[0, 9], [9, 9]])` returns `1` — touching seals merge.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
