## Idea
Root-to-leaf sums over index arrays keep the running-total discipline inside a small signature.

## Why it is original
The kindred wording, the leaf-only rule and the examples were written for this pack.

## Patterns used
Descent carrying a running total to leaf checks.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty, singleton, skewed, negative and stray shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
