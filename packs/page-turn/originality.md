## Idea
FIFO page faults — the folio forgets what arrived first, not what was used last.

## Why it is original
The folio wording, the oldest-first rule and the case set were written for this pack.

## Patterns used
Bounded FIFO set with an eviction cursor.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover full churn, repeat streams and single-slot folios (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n*f) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
