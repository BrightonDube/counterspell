## Idea
A keeper records tide heights each hour. Report the first hour whose reading is strictly lower than the reading before it, or a sentinel when the tide never drops.

## Why it is original
The keeper-and-tide-table framing and every sentence here were written for this pack. The first-strict-decrease contract is a general scan, expressed in new wording with new examples.

## Patterns used
A single forward scan that compares neighbouring readings and an early return sentinel.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; readings are non-negative so no "integer" wording is needed (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Eleven hidden cases were derived by hand and cover the required shapes (AU-5, AU-6). Two ghosts are realistic comparison and indexing mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. A first-descent scan is a general technique, used without borrowed wording.
