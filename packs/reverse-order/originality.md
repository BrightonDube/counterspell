## Idea
Reversing chain order without pointers asks for either a collected walk or a reversed pointer pass.

## Why it is original
The wording and hop-around examples were written for this pack.

## Patterns used
Pointer-style traversal reversed; a collected list then a flip.

## Checklist answers
The wording is my own (AU-1). Two `list[int]` params return `list[int]` (AU-2). Constraints match the schema (AU-3). The hop example is one line (AU-4). Hidden cases cover empty, singleton, woven and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
