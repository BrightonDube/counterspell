## Idea
Shortest path over a priced web — the greedy walk must hold the true ledger.

## Why it is original
The ember-road wording, the both-ways rule and the doubled-arc clause were written for this pack.

## Patterns used
Lazy Dijkstra over the aligned lists.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover stranded nodes, ties, doubled arcs, big tolls and lone webs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O((n + arcs) log n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
