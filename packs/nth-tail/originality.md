## Idea
Reading nth-from-end over a one-way chain asks for two walks or a collected list — pointer arithmetic cannot help.

## Why it is original
The wording and examples were written for this pack.

## Patterns used
Two-pass or collected-list traversal.

## Checklist answers
The wording is my own (AU-1). Two `list[int]` params plus `int` return `int` (AU-2). Constraints match the schema (AU-3). The examples take one line each (AU-4). Hidden cases cover empty, singleton, out-of-range, stray and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space for the two-walk scheme (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
