## Idea
Placing one sigil per row with column and diagonal bans is the classic constraint-search walk — and the count, not a witness, is wanted.

## Why it is original
The sigil wording, the count-the-placements contract and the bound were written for this pack.

## Patterns used
Row-by-row backtracking pruning columns and both diagonals.

## Checklist answers
The wording is my own (AU-1). The function uses an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty, trivial, impossible and large boards (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states the bounded permutation walk (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
