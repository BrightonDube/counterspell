## Idea
Interval partitioning — the answer is the deepest overlap, found by a sweep.

## Why it is original
The keeper wording, the shared-mark collision rule and the cases were written for this pack.

## Patterns used
Event sweep, starts before ends at a shared mark.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover touching, nested, duplicate and disjoint holds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
