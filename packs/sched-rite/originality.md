## Idea
Precedence-graph acyclicity — the serialisability check behind schedules.

## Why it is original
The binding-rule wording, the cycle verdict and the graph cases were written for this pack.

## Patterns used
DFS with a visiting colour, or topological peeling.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover self-loops, two-cycles, long cycles and DAG diamonds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rites + rules) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
