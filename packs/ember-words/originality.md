## Idea
Dictionary word break — every prefix asks whether some ember ends exactly at it.

## Why it is original
The ember wording, the comma-joined lexicon and the trapped-s cases were written for this pack.

## Patterns used
Prefix reachability over word ends.

## Checklist answers
The wording is my own (AU-1). The function takes two `str` inputs and returns `int` (1 for splits, 0 otherwise) (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover trapped letters, overlapping embers, empty text and repeated ember reuse (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len × embers) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
