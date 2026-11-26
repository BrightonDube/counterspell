## Idea
The eccentricity of node zero — a breadth count rather than a reach count.

## Why it is original
The span wording, the out-of-reach rule and the hop-count contract were written for this pack.

## Patterns used
Breadth-first layering from node zero.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover bare webs, stars, chains, far isles and quick detours (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
