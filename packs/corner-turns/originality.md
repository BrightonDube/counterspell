## Idea
Left-turn counting along a polyline — signed cross products, no floats.

## Why it is original
The route wording, the left-only count and the case set were written for this pack.

## Patterns used
Cross-product sign at each interior vertex.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover straights, squares, reversals and short routes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(points) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
