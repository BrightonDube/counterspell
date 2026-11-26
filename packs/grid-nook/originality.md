## Idea
A flood on a bricked grid — the traversal is local and four-way.

## Why it is original
The nook wording and the top-left rule were written for this pack.

## Patterns used
Four-way flood with a visited ledger on a small grid.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover bricked starts, funnels, stripes and full grilles (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows × cols) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
