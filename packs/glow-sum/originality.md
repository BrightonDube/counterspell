## Idea
Summing reachable marks over a serialised tree keeps traversal and the stray-cell rule in one small task.

## Why it is original
The glow wording, the stray-value example and the reachability rule were written for this pack.

## Patterns used
Depth-first walk accumulating a total.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The stray-value example is one line (AU-4). Hidden cases cover empty, singleton, skewed, negative and stray shapes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
