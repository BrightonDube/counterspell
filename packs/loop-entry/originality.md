## Idea
Finding where a knotted chain closes is the classic slow/fast pointer meeting — over index arithmetic.

## Why it is original
The knotted-lantern wording and examples were written for this pack.

## Patterns used
Slow/fast traversal or a visited set with the first re-visited index.

## Checklist answers
The wording is my own (AU-1). `list[int]` returns `int` (AU-2). Constraints match the schema (AU-3). The examples name each loop in one line (AU-4). Hidden cases cover self-loops, distant entries, no loops and unreachable loops (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
