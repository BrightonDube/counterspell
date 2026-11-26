## Idea
LRU page faults — a hit renews the page's freshness before the next eviction.

## Why it is original
The freshness wording, the touch-renewal rule and the case set were written for this pack.

## Patterns used
Recency-ordered bounded set.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover touch renewals, deep re-use and single-slot folios (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n*f) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
