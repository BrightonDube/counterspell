## Idea
Sentinel scanning — the marked row is data, not a missing row.

## Why it is original
The ledger-mark wording, the `-1` sentinel contract and the case set were written for this pack.

## Patterns used
Predicate count over rows.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[list[int]]` and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty ledgers, keys of -1 confusion and all-marked tables (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(rows) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
