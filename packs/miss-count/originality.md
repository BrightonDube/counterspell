## Idea
Direct-mapped cache counting — the slot is set by the mark, the tag is what stays.

## Why it is original
The ward wording, the tag contract and the case set were written for this pack.

## Patterns used
Slot array with tag compare.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover tag collisions, cold starts and all-hit streams (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
