# The Alloc Rite

## Task
The vault holds fixed slots in order — each request takes the first slot that fits and shrinks it by the amount taken. Call `alloc_rite(blocks, reqs)` and return how many requests find a home.

## Input
`blocks` lists slot sizes (`0` to `60`); `reqs` lists request sizes (`0` to `60`).

## Output
The count of requests granted by first-fit.

## Examples
- `alloc_rite([10, 4, 8], [6, 4, 8, 2, 9])` returns `4` — `9` finds no slot.
- `alloc_rite([4, 4, 4], [4, 4, 4, 4])` returns `3`.
- `alloc_rite([], [1, 2])` returns `0`.

## Constraints
- `0 <= len(blocks), len(reqs) <= 16` with `0 <= size <= 60`
