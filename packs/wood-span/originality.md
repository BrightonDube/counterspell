## Idea
Kruskal's span over a priced web — the greedy pick is provably correct here.

## Why it is original
The span wording, the doubled-arc clause and the unjoinable sentinel were written for this pack.

## Patterns used
Sorted-arc union-find with a running toll.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover rings, cut-off isles, doubled arcs, loops, ties and lone nodes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(arcs log arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
