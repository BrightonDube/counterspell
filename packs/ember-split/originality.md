## Idea
An equal split is a subset hitting half the total — the backtracking picks embers rather than positions.

## Why it is original
The ember-half wording, the non-empty rule and the unordered count were written for this pack.

## Patterns used
Subset-sum search at half the total, halved for unorderedness.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover odd totals, empty piles, all-zero piles and repeats (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(2^n) worst case within the bound (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
