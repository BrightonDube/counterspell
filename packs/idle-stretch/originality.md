## Idea
Longest uncovered run inside a bounded watch.

## Why it is original
The watch wording, the bounded idle run and the cases were written for this pack.

## Patterns used
Coverage sweep over marks 0..t.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty watches, full holds, out-of-range seals and edge runs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(t) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
