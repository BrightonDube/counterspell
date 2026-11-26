## Idea
Deadlock-order check — shared ward pairs must keep one global direction.

## Why it is original
The ward wording, the pairwise-direction rule and the case set were written for this pack.

## Patterns used
Pairwise direction extraction per row, consistency check across rows.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover disjoint wards, three-way agreement and repeated wards in one row (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows * wards^2) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
