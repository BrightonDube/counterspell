## Idea
The classic take-or-skip row — the best haul through position i picks between taking mark i (skipping its neighbour) and skipping it.

## Why it is original
The calm-gaps wording, the adjacency break and the empty-haul rule were written for this pack.

## Patterns used
Two running bests: best-through-take and best-through-skip.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty rows, single marks, ties and alternating runs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
