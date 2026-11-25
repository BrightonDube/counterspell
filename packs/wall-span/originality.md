## Idea
Shelf blocks of different heights stand shoulder to shoulder; the largest canvas that can stretch across a level run is the answer.

## Why it is original
The wall-and-canvas scenario and its wording were written for this pack. The examples and constraints are new and small enough to check by hand.

## Patterns used
The monotonic-stack pass that resolves each block's widest span when a shorter block closes it.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 1 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, plateau, dip, peak and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Largest-rectangle scanning is used as a technique only.
