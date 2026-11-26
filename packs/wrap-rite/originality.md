## Idea
Ring-buffer overwrite counting — a resident mark costs nothing, a stranger on a full ring evicts.

## Why it is original
The ring wording, the no-op rule and the case set were written for this pack.

## Patterns used
Bounded FIFO set — identical mechanics to FIFO paging on a different stage.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover warm repeats, full churn and single-slot rings (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n*b) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
