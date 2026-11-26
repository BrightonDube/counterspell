## Idea
The least fusion cost is the two-smallest meld a min-heap drives at every step — Huffman's old move in an ember yard.

## Why it is original
The meld-cost wording, the greedy two-smallest rule and the single-pile answer were written for this pack.

## Patterns used
A min-heap repeatedly fusing the two smallest piles.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty and single piles, equal sizes, skewed piles and larger counts (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) heap work (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
