## Idea
A sorted row of lantern heights is searched for the first lantern at least as tall as a mark; when none qualifies, the answer is the row's length.

## Why it is original
The warden's height check, the mark vocabulary and the out-the-end answer were written for this pack.

## Patterns used
Lower-bound binary search with ties and an end-of-list answer.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema (AU-3). The worked example explains a boundary hit in under four lines (AU-4). Twelve hidden cases cover empty, all-below, all-clear, ties and maximum shapes (AU-5, AU-6). Three ghosts carry hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(log n) and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
