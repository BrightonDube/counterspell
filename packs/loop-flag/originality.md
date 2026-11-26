## Idea
Flagging a knotted chain forces a memory or a visited set — the classic cycle check over index pointers.

## Why it is original
The knotted-lantern wording and the unreachable-loop examples were written for this pack.

## Patterns used
Visited-set traversal over an index array.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples name the cycle in one line (AU-4). Hidden cases cover self-loops, distant loops, unreachable loops and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
