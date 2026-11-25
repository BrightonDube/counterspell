## Idea
Two vault ledgers record amounts by key, and a key may be written several times in one ledger. Total each key per ledger, then count the keys whose totals agree across both ledgers.

## Why it is original
The paired-ledger join, the repeated-key totals and the wording are new for this pack.

## Patterns used
Summing rows into a per-key map and joining the two maps on shared keys.

## Checklist answers
The wording is my own (AU-1). The function uses `list[list[int]]` params and `int` only (AU-2). Constraints match the schema; keys and amounts are non-negative (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Twelve hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic join mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n+m) time and O(k) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Keyed joins are a general technique.
