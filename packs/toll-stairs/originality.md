## Idea
Cheapest ascent — each stair's best toll feeds the two stairs above it.

## Why it is original
The toll-stairs wording, the free start and the above-the-last finish were written for this pack.

## Patterns used
Prefix min-DP with a two-step window.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty and single stairs, zero tolls and alternating dear/cheap runs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
