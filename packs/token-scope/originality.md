## Idea
Fail-closed capability check — full coverage plus liveness, both required.

## Why it is original
The token wording, the boundary-expiry rule and the case set were written for this pack.

## Patterns used
Subset test plus liveness bound.

## Checklist answers
The wording is my own (AU-1). The function takes two `list[int]`s and two `int`s and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover boundary expiry, dead tokens and empty requirements (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(r*g) worst case (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
