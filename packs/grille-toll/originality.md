## Idea
Fewest cells across a blocked grille — the min-plus table with dead cells.

## Why it is original
The toll wording, the cell-count rule and the walled lanes were written for this pack.

## Patterns used
Two-axis min table with unreachable cells.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover walled lanes, blocked ends, long detours and full blockage (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows × cols) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
