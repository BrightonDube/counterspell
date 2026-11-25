## Idea
A tide watch rings whenever a reading out-tops everything in its look-back window — a streaming max comparison.

## Why it is original
The tide-gauge scenario and its wording were written for this pack. The examples and limits are new and small enough to check by hand.

## Patterns used
Sliding-window streaming condition — each element judged against the window that precedes it.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 1 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, strict-boundary, monotone, tied and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n·w) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Streaming window comparison is used as a technique only.
