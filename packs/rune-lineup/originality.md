## Idea
Counting permutations under an adjacency ban is the placement-search shape — position by position, with a used-mark ledger.

## Why it is original
The lineup wording, the distinct-candles rule and the tiny bound were written for this pack.

## Patterns used
Permutation backtracking with a used-position ledger and the adjacency test.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover single candles, all-equal marks, repeats and longer rows (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n!) worst case within the bound (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
