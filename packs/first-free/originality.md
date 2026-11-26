## Idea
The smallest uncovered integer mark — a mex over merged stretches.

## Why it is original
The bare-mark wording, the from-zero rule and the cases were written for this pack.

## Patterns used
Coverage walk or sorted merge to the first gap.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover bare zero, full coverage, single gaps and seals away from zero (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(covered) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
