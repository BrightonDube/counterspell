## Idea
A rise-then-fall ridge is searched for its single crest by comparing each probe with its right neighbour.

## Why it is original
The torch-ridge scenario, the strict-rise-then-fall contract and the examples were written for this pack.

## Patterns used
Binary search on a unimodal sequence; the comparison with the next mark picks the half holding the crest.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and returns `int` (AU-2). Constraints match the schema — the rise-then-fall shape is documented in the statement (AU-3). The worked example explains the crest probe in under four lines (AU-4). Twelve hidden cases cover crest at each end, minimal ridges and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
