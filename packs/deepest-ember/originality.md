## Idea
A warden tracks how deep nested layers ever reach in a rune inscription, where stray closing runes are ignored.

## Why it is original
The warden-and-ember scenario and its wording were written for this pack. The examples and constraints are new and small enough to check by hand.

## Patterns used
A running depth counter with a floor at zero — the counting form of a stack.

## Checklist answers
The wording is my own (AU-1). The function uses `str` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 2 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, stray, unclosed and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Depth tracking is used as a technique only.
