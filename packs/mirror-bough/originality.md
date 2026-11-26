## Idea
Mirror checking over index arrays exercises paired-subtree descent without real pointers.

## Why it is original
The bough wording, the paired-node contract and the examples were written for this pack.

## Patterns used
Simultaneous descent over two mirrored branches.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty, singleton, mirrored, skewed and near-miss shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
