## Idea
Inner-join cardinality — duplicate keys multiply, empty sides vanish.

## Why it is original
The ledger wording, the multiplication rule and the case set were written for this pack.

## Patterns used
Nested-loop join counting, or key-bucket multiplication.

## Checklist answers
The wording is my own (AU-1). The function takes two `list[list[int]]`s and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty sides, key collisions and one-sided keys (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(a*b) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
