## Idea
A sorted toll ledger is asked how many entries sit strictly below a toll — a lower-bound count, not a lookup.

## Why it is original
The toll-ledger scenario, the strict-below wording and the examples were written for this pack.

## Patterns used
Lower-bound binary search where the bound itself is the answer.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example explains a mid-list count in under four lines (AU-4). Twelve hidden cases cover empty, all-below, none-below, repeats and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
