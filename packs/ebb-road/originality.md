## Idea
Bellman-Ford over a one-way web with negative prices — the teaching point is distinguishing unreachable from unbounded.

## Why it is original
The ebb wording, the one-way rule, the ring sentinel and the pay-back cases were written for this pack.

## Patterns used
n-1 relaxations plus a marked-reach pass for negative rings.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover negative arcs, rings that reach and rings that do not, stranded targets and self-loops (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n × arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
