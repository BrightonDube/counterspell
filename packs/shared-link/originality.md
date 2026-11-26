## Idea
Two walks over one pointer vault meet at a shared tail; the first common index is found by set membership, not lockstep position.

## Why it is original
The shared-vault wording and the two-heads design were written for this pack.

## Patterns used
Visited-set intersection over two walks.

## Checklist answers
The wording is my own (AU-1). `list[int]` plus two `int` params return `int` (AU-2). Constraints match the schema (AU-3). The examples explain each meet or miss in one line (AU-4). Hidden cases cover misses, immediate shares, crossed walks and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n+m) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
