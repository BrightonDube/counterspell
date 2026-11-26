## Idea
Post-partition quorum counting — drop failed links, find isles, count strict majorities.

## Why it is original
The isle wording, the index-based failure list and the case set were written for this pack.

## Patterns used
Union-find or BFS over surviving links; strict-majority test per component.

## Checklist answers
The wording is my own (AU-1). The function takes `int`, `list[list[int]]`, `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover even splits, lone towers and no-failure wards (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + links) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
