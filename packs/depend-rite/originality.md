## Idea
Functional-dependency verification — the X-to-Y consistency check.

## Why it is original
The settling wording, the 1/0 verdict contract and the split cases were written for this pack.

## Patterns used
First-seen map, then consistency check.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty ledgers, agreeing repeats and late splits (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
