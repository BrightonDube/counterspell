## Idea
A sorted stairwell is asked for the tallest step that still fits under a reach — an upper-bound lookup returning a value, not a position.

## Why it is original
The stairwell scenario, the reach wording and the -1 contract were written for this pack.

## Patterns used
Upper-bound binary search; the answer is the mark just before the bound.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example explains a mid-stair answer in under four lines (AU-4). Twelve hidden cases cover empty, all-above, all-below, repeats and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
