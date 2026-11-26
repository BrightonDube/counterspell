## Idea
The gap turns non-adjacency into a tunable constraint — the same place/skip search with a wider forced skip.

## Why it is original
The wall wording and the tunable gap were written for this pack.

## Patterns used
Place/skip recursion skipping `gap + 1` posts after each sentry.

## Checklist answers
The wording is my own (AU-1). The function uses three `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover zero sentries, exact fits, wide gaps and the bounds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states bounded place/skip recursion (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
