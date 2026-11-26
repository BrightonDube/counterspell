## Idea
Linearly decayed ranking gain — a weighted prefix sum.

## Why it is original
The favour-decay framing, the `(k - i)` weight and the case set were written for this pack.

## Patterns used
Index-weighted accumulation over a bounded prefix.

## Checklist answers
The wording is my own (AU-1). The function takes `list[int]` and `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover shallow strips, zero depth and dead tallies (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(min(k, n)) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
