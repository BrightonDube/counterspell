## Idea
Windowed partition DP — streaming chunking meets dynamic programming.

## Why it is original
The toll framing, the spread fee and the case set were written for this pack.

## Patterns used
Prefix DP over bounded window ends, tracking running min/max.

## Checklist answers
The wording is my own (AU-1). The function takes `list[int]`, `int`, `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover clamped widths, flat fees and even streams (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len * w) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
