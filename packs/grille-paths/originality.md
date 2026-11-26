## Idea
Right-or-down counting on a bricked grille is the bounded state walk — each cell branches into at most two moves.

## Why it is original
The grille wording, the brick rule and the tiny board were written for this pack.

## Patterns used
Path counting with a visited-free recursion — right and down only, so no marks are needed.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover single cells, bricked starts, funnels and larger grilles (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(2^(r+c)) worst case within the bound (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
