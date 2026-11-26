## Idea
Tarjan's low-link over a multigraph — the parallel-copy rule is what makes this pack distinct.

## Why it is original
The bridge wording, the doubled-arc clause and the self-loop rule were written for this pack.

## Patterns used
Depth-first stamps with low-link bookkeeping, tracking the entry arc index.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover forests, rings, tails on rings, doubled arcs, loops and split webs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
