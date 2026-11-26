## Idea
Sliding-window mark density over a small alphabet stream.

## Why it is original
The scrying-window framing, the short-stream clamp and the case set were written for this pack.

## Patterns used
Sliding window with an increment/decrement counter.

## Checklist answers
The wording is my own (AU-1). The function takes `list[int]`, `int`, `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover closed windows, oversized windows and absent marks (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len tokens) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
