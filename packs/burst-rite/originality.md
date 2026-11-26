## Idea
First-come-first-served waiting-time total — the engine may idle, ties keep listing order.

## Why it is original
The engine wording, the tie contract and the case set were written for this pack.

## Patterns used
Event sweep with a running clock.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty queues, idle gaps and same-arrival ties (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(j log j) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
