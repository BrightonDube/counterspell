## Idea
A bounded subset count is the take/leave search with a running weight — the satchel version of subset-sum.

## Why it is original
The satchel wording, the empty-subset rule and the bounded marks were written for this pack.

## Patterns used
Take/leave recursion carrying the remaining capacity.

## Checklist answers
The wording is my own (AU-1). The function uses `int_array` and `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty piles, tight and loose caps and repeats (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(2^n) worst case with pruning (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
