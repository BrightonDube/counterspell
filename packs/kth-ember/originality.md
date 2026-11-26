## Idea
Rank selection is the depth-k pop of a min-heap, stated as a single mark query.

## Why it is original
The rank wording, the repeat-rank rule and the short-pile answer were written for this pack.

## Patterns used
Ordering then an index pick; equally k pops from a min-heap.

## Checklist answers
The wording is my own (AU-1). The function uses `int_array` and `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover short piles, k of zero, repeats, negatives and a full-pile pick (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) ordering or O(k log n) with a heap (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
