## Idea
Collecting marks in chain order shows whether the learner truly walks the pointers or trusts the array order.

## Why it is original
The lantern wording and the hop-around examples were written for this pack.

## Patterns used
Pointer-style traversal with a collected result.

## Checklist answers
The wording is my own (AU-1). The function uses two `list[int]` params and returns `list[int]` (AU-2). Constraints match the schema (AU-3). The hop-around example is one line (AU-4). Hidden cases cover empty, singleton, reordered and stray shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) output space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
