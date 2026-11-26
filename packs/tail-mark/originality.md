## Idea
Reading the chain's tail exercises the same traversal, with stray cells that sit past the tail.

## Why it is original
The lantern-tail wording and the stray-cell examples were written for this pack.

## Patterns used
Pointer-style traversal; the last visited cell is the answer.

## Checklist answers
The wording is my own (AU-1). The function uses two `list[int]` params and returns `int` (AU-2). Constraints match the schema (AU-3). The stray-cell example is one line (AU-4). Hidden cases cover empty, singleton, stray and reordered shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
