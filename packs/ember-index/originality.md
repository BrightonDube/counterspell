## Idea
A sorted rack of distinct ember marks is searched for one exact mark; the answer is its shelf position or -1.

## Why it is original
The ember-rack scenario and its wording were written for this pack. Examples and limits are new and small enough to check by hand.

## Patterns used
Exact-match binary search over a sorted, distinct integer array.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema, including `sorted` and `unique` (AU-3). The worked example calls the function on a sample and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover empty, edges, negatives, absent values and maximum shapes (AU-5, AU-6). Three ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Binary search is used as a technique only.
