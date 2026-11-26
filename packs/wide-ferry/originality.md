## Idea
The widest path — maximize the weakest arc instead of minimizing the sum.

## Why it is original
The ferry wording, the capacity-as-weight reading and the stranded sentinel were written for this pack.

## Patterns used
Max-heap maximin walk.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]`, a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover stranded starts, thin caps, doubled arcs, direct-vs-detour and lone nodes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O((n + arcs) log n) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
