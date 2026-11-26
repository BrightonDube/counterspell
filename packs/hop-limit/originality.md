## Idea
Breadth-limited reachability — a BFS that stops when the hop budget dies.

## Why it is original
The packet-life wording, the ttl contract and the case set were written for this pack.

## Patterns used
Depth-bounded BFS.

## Checklist answers
The wording is my own (AU-1). The function takes `int`, `list[list[int]]`, `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover dead packets, islands and fan-out (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + links) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
