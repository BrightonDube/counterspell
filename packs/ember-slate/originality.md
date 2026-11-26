## Idea
The longest strictly climbing subsequence — every mark asks which earlier glows it can out-glow.

## Why it is original
The slate wording, the strict-climb rule and the tied-glow cases were written for this pack.

## Patterns used
O(n²) per-position best.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty slates, falling rows, tied glows and long climbs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len²) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
