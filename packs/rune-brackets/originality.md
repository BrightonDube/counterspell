## Idea
A lintel is scratched with three kinds of bracket runes, and the carving holds only when every closing rune meets the opener still waiting beneath it.

## Why it is original
The lintel-rune scenario and its wording were written for this pack. The examples and constraints are new and small enough to check by hand.

## Patterns used
The classic last-in, first-out check for nested brackets — a stack of openers.

## Checklist answers
The wording is my own (AU-1). The function uses `str` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 2 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, sound, crossed, trailing-opener and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Nested-bracket checking is used as a technique only.
