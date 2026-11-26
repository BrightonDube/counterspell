## Idea
Subset counting by bitmask enumeration — the mask is the point, not pruning.

## Why it is original
The candle-lighting wording, the cap semantics and the ghost set were written for this pack.

## Patterns used
Bitmask enumeration over 2^n lightings.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty sets, zero caps, zero flames and tight bounds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(2^n * n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
