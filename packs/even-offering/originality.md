## Idea
The smallest feasible heaviest-share cap is searched over the answer space; a cap is feasible when greedy runs under it need at most `parts` stretches.

## Why it is original
The offering-bowl scenario, the keeper vocabulary and the examples were written for this pack.

## Patterns used
Answer-space binary search plus a greedy feasibility scan.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` (AU-2). Constraints match the schema and keep `parts <= len(offers)` so a split always exists (AU-3). The worked example shows both best splits in under four lines (AU-4). Twelve hidden cases cover one keeper, per-bowl splits, skewed rows and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log s) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
