## Idea
Order checking over an index-array tree turns the range-bounds idea into a small structural task.

## Why it is original
The ward wording, the strict-marks rule and the examples were written for this pack.

## Patterns used
Descent with tightening lower and upper bounds.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty, singleton, local-violation, deep-violation and duplicate shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
