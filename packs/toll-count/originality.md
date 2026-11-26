## Idea
A threshold count over the toll ledger.

## Why it is original
The cap wording and the inclusive-bound rule were written for this pack.

## Patterns used
Filtered count over the aligned lists.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty lists, the inclusive bound, big caps and mismatched lengths (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
