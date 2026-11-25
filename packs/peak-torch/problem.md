# The Peak Torch

## Task
A ridge line of torch heights climbs for a while, then falls — every step strictly rises or strictly falls, never level, and exactly one crest exists. Call `peak_torch(heights)` and return the crest's position.

## Input
A list of heights with at least two marks: a strictly rising run followed by a strictly falling run, one crest between them.

## Output
The zero-based position of the crest.

## Examples
- `peak_torch([1, 4, 7, 9, 6, 3])` returns `3`.
- `peak_torch([2, 5, 1])` returns `1`.
- `peak_torch([9, 1])` returns `0`.

## Constraints
- `2 <= len(heights) <= 40`
- heights are integers, `0 <= height <= 99`
- there is exactly one crest: heights rise strictly to it, then fall strictly
