## Idea
Interleaving check — a two-strand reachability table where each prefix pair is fed by whichever strand matches next.

## Why it is original
The braid wording, the exact-length rule and the strand cases were written for this pack.

## Patterns used
Two-axis reachability over strand prefixes.

## Checklist answers
The wording is my own (AU-1). The function takes three `str` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty strands, length mismatches, doubled letters and exact interleaves (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len × len) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
