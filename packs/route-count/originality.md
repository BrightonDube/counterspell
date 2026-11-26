## Idea
Reachability count on an undirected link graph — a breadth walk from the source tower.

## Why it is original
The tower wording, the exclusion of the source and the case set were written for this pack.

## Patterns used
BFS or union-find over a tiny graph.

## Checklist answers
The wording is my own (AU-1). The function takes `int` and `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty graphs, lone towers and disconnected wards (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + links) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
