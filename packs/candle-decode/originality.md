## Idea
A digit-line decoding count — each position extends a single or a pair, with zero as the trap.

## Why it is original
The candle wording, the zero rules and the pair bound were written for this pack.

## Patterns used
Prefix DP over one- and two-glyph steps.

## Checklist answers
The wording is my own (AU-1). The function takes a `str` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover zeros, pairs over 26, runs of ones and long lines (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(len) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
