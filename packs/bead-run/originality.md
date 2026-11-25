## Idea
A strand of two bead kinds is measured for its longest unbroken run of one kind.

## Why it is original
The bead-strand scenario and its wording were written for this pack. The examples and limits are new and small enough to check by hand.

## Patterns used
Run-length scanning — carry the current run length and keep the maximum.

## Checklist answers
The wording is my own (AU-1). The function uses `str` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 1 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, tied, edge-placed, alternating and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Run-length scanning is used as a technique only.
