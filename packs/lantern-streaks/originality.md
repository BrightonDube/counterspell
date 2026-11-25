## Idea
A row of lanterns ends a street. Each lantern is either lit or dark. Count how many lanterns are lit in a row, reading backward from the last lantern, until a dark lantern stops the run.

## Why it is original
The scenario — a lit run counted from the end of the row — and its wording were written for this pack. The examples and constraints are new and small enough to check by hand.

## Patterns used
A single backward scan that stops at the first dark mark, and a running count.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema and no negatives are allowed, so no "integer" wording is needed (AU-3). The worked example calls the function with sample 1 and explains it in under four lines (AU-4). Twelve hidden cases were derived by hand and cover every required shape (AU-5, AU-6). Two ghosts are realistic mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. General run-counting patterns were used as techniques only.
