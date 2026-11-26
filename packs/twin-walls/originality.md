## Idea
Two-colouring a sparse web is the canonical bipartite check — a traversal that assigns camp and watches for clashes.

## Why it is original
The two-camp wording, the self-loop rule and the disconnected-web clause were written for this pack.

## Patterns used
Flood with a two-colour ledger, restarted per isle.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover odd rings, even rings, self-loops, split webs and repeated arcs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + arcs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
