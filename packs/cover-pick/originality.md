## Idea
Greedy set cover over bitmasks — bits meet greedy simulation.

## Why it is original
The talisman framing, the out-of-range bit rule and the case set were written for this pack.

## Patterns used
Popcount maximisation over a shrinking uncovered set.

## Checklist answers
The wording is my own (AU-1). The function takes `int` and `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover ties, dead masks and out-of-range bits (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(masks * n picks) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
