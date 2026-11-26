## Idea
Per-sender high-water mark — a nonce at or under the mark is a replay, not a fresh message.

## Why it is original
The seal-wording, the per-sender counters and the boundary cases were written for this pack.

## Patterns used
Streamed max tracking keyed by sender.

## Checklist answers
The wording is my own (AU-1). The function takes `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover cross-sender counters, equal nonces and descending streams (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(msgs) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
