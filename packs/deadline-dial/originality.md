## Idea
The deadline dial is the classic keep-or-drop schedule: walk deadlines in order and drop the longest held job whenever the dial overruns.

## Why it is original
The dial wording, the flat-pair encoding and the keep-or-drop rule were written for this pack.

## Patterns used
Sorting by deadline plus a max-heap of held durations — drop the longest on overrun.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dials, all-fitting and all-clashing jobs, ties and drops (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n log n) sort plus heap work (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
