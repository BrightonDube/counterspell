## Idea
Non-adjacent placement is the canonical spacing constraint — place a guard, then skip a cell.

## Why it is original
The ward-row wording and the placement count were written for this pack.

## Patterns used
Place-or-skip recursion with a forced gap after each guard.

## Checklist answers
The wording is my own (AU-1). The function uses two `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover zero guards, full rows, impossible spreads and the bounds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states a bounded recursion or closed form (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
