## Idea
A rotated sorted edge hides its smallest mark at the single break; the break is found by comparing probes with the far end.

## Why it is original
The cliff-edge scenario and its examples were written for this pack.

## Patterns used
Binary search for the rotation point; the mark after the last drop is the minimum.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and returns `int` (AU-2). Constraints match the schema, including `unique` (AU-3). The worked example explains an unrotated edge in under four lines (AU-4). Twelve hidden cases cover single marks, unrotated, half-rotated and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
