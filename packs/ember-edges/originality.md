## Idea
A sorted rack with repeats is asked for both ends of one mark's run — two boundary searches joined into a pair.

## Why it is original
The paired-edges contract, the rack wording and the examples were written for this pack.

## Patterns used
Two lower-bound walks: first index of the target and first index past it.

## Checklist answers
The wording is my own (AU-1). The function returns `list[int]` (AU-2). Constraints match the schema (AU-3). The worked example explains a repeated run in under four lines (AU-4). Twelve hidden cases cover empty, single, full-run, absent and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
