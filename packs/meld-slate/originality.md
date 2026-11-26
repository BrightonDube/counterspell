## Idea
A two-way meld is the manual version of a merge-heap: two ordered fronts, one ordered outflow.

## Why it is original
The slate wording, the repeat-keeping rule and the empty-slate cases were written for this pack.

## Patterns used
Two indices walking both fronts — the merge half of merge sort.

## Checklist answers
The wording is my own (AU-1). The function uses two `int_array` inputs and returns `list[int]` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty slates, repeats, interleaving and single fronts (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + m) time and space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
