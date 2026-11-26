## Idea
Lowest shared ancestry over index arrays turns parent-climbing into a small reachable-set task.

## Why it is original
The shared-ancestor wording, the index answer and the unreachable-node rule were written for this pack.

## Patterns used
Parent map construction plus an ancestor climb.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty, sibling, root, deep and unreachable shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
