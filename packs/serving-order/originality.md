## Idea
A line of marks is served from the front, and the record of who left in what order is the answer.

## Why it is original
The cauldron-line scenario and its wording were written for this pack. The examples and constraints are new and small enough to check by hand.

## Patterns used
A first-in, first-out simulation that records each successful serving.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` only (AU-2). Constraints match the schema (AU-3). The worked example calls the function with sample 1 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, over-served, interleaved and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values or a crash class (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Queue simulation is used as a technique only.
