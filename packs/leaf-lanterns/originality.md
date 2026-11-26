## Idea
Counting leaves over a serialised tree keeps pointer reasoning inside plain arrays.

## Why it is original
The lantern-grove wording, the reachability rule and the stray-cell example were written for this pack.

## Patterns used
Depth-first walk over child index arrays.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The stray-cell example is one line (AU-4). Hidden cases cover empty, singleton, skewed, full and stray shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
