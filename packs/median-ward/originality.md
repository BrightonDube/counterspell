## Idea
The two-heap median keeps a running pivot; scaling by two keeps the answer integral at even counts.

## Why it is original
The doubled-median contract, the even-count sum rule and the empty answer were written for this pack.

## Patterns used
Ordering then a middle read; equally a two-heap balance.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty wards, singletons, even splits, negatives and repeats (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) ordering or O(1) extra with a heap view (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
