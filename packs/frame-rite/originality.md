## Idea
Optimal page replacement — clairvoyant eviction by farthest next use, leftmost on ties.

## Why it is original
The frame wording, the leftmost-tie rule and the case set were written for this pack.

## Patterns used
Lookahead eviction with a next-use scan.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover never-again ties, warm repeats and full churn (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n^2) lookahead (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
