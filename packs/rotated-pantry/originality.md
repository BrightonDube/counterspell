## Idea
A rotated sorted shelf must still answer an exact-position lookup in log time — each probe keeps one ordered half.

## Why it is original
The pantry-rotation scenario and its examples were written for this pack.

## Patterns used
Binary search on a rotated sorted array; at each probe one half is provably ordered.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` (AU-2). Constraints match the schema, including `unique` (AU-3). The worked example explains a cross-break hit in under four lines (AU-4). Twelve hidden cases cover unrotated, half-rotated, empty, absent and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
