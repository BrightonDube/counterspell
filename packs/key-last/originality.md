## Idea
Last-write-wins lookup — row order carries the history.

## Why it is original
The ledger-write wording, the absent-key sentinel and the ordering cases were written for this pack.

## Patterns used
Linear scan keeping the latest hit.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover absent keys, first-vs-last writes and key zero (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
