## Idea
Draining a priority queue with a stable (urgency, id) order is the minimal heap simulation that still forces a comparator.

## Why it is original
The flat-pair encoding, the id tie-break and the drain wording were written for this pack.

## Patterns used
Sorting pairs by a two-key comparator; equally repeated min-heap pops.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `list[int]` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty queues, ties, equal ids spread over urgencies and reversed arrivals (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) ordering (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
