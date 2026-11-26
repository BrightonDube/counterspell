## Idea
Strict-majority quorum test — a half-council split must fail closed.

## Why it is original
The keeper wording, the strict bound and the tie cases were written for this pack.

## Patterns used
Integer floor division for the majority threshold.

## Checklist answers
The wording is my own (AU-1). The function takes two `int`s and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover exact ties, empty councils and single keepers (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(1) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
