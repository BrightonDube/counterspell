## Idea
Shortest-path counting — the ledger holds both cost and multiplicity.

## Why it is original
The chain-toll wording, the tie semantics and the doubled-arc clause were written for this pack.

## Patterns used
Dijkstra with a parallel count ledger.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover ties, stranded targets, doubled arcs, a single node and big tolls (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O((n + arcs) log n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
