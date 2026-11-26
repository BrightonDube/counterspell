## Idea
Distinct-mark count over a retry log — deduplication by rite id, not sequence.

## Why it is original
The courier wording, the seq/rite split and the case set were written for this pack.

## Patterns used
Set membership over a stream.

## Checklist answers
The wording is my own (AU-1). The function takes `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty logs, all-retries and out-of-order sequences (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
