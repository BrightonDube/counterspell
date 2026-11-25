## Idea
The smallest brew pace that meets a deadline is searched over the answer space rather than the row itself.

## Why it is original
The brew-pace scenario, the ceil-hours model and the examples were written for this pack.

## Patterns used
Answer-space binary search with a monotone feasibility check.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` (AU-2). Constraints match the schema and keep `hours >= len(brews)` so a pace always exists (AU-3). The worked example shows the checked sums in under four lines (AU-4). Twelve hidden cases cover single brews, tight deadlines, loose deadlines and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log m) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
