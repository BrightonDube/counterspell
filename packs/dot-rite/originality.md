## Idea
Sparse dot product over position-keyed marks — a merge on shared keys.

## Why it is original
The scroll wording, the last-mark-wins rule and the case set were written for this pack.

## Patterns used
Dictionary merge or two-pointer walk over keyed rows.

## Checklist answers
The wording is my own (AU-1). The function takes two `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty scrolls, duplicates and disjoint positions (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len a + len b) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
