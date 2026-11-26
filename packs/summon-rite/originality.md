## Idea
Subset-sum existence is the smallest honest backtracking problem: branch take-or-skip over every mark.

## Why it is original
The summon wording, the empty-choice rule and the bounded marks were written for this pack.

## Patterns used
Depth-first search over take/skip branches with early exit.

## Checklist answers
The wording is my own (AU-1). The function uses `int_array` and `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty piles, zero targets, negatives and unreachable marks (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(2^n) worst case with pruning (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
