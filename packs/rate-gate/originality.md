## Idea
Token-bucket admission — refill on elapsed ticks, spend on admission, fail closed on empty.

## Why it is original
The seal-bucket wording, the refill-cap rule and the case set were written for this pack.

## Patterns used
Event sweep with a capped accumulator.

## Checklist answers
The wording is my own (AU-1). The function takes `list[list[int]]`, `int`, `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover dry buckets, same-tick bursts and over-capacity asks (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(events) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
