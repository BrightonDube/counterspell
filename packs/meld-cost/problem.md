# The Meld Cost

## Task
Melding two piles costs the sum of their sizes, and the melded pile can meld on. Call `meld_cost(sizes)` and return the least total cost to fuse every pile into one.

## Input
`sizes` holds pile sizes; each is positive. One pile or no piles cost `0`.

## Output
The minimum total cost of all the melds.

## Examples
- `meld_cost([1, 2, 3])` returns `9` — `1+2` costs `3`, then `3+3` costs `6`.
- `meld_cost([4, 4, 4, 4])` returns `32` — meld pairs first: `8 + 8 + 16`.
- `meld_cost([])` returns `0`.

## Constraints
- `0 <= len(sizes) <= 40`, `1 <= sizes[i] <= 99`
