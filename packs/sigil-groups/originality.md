## Idea
Every rune marking carries a sigil made of the sum of its digits. Group the markings by sigil and report the size of the largest group.

## Why it is original
The sigil grouping, the digit-sum signature and all wording are new for this pack, and the cases were built by hand.

## Patterns used
A signature key per item and a frequency table over keys.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; markings are non-negative (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Eleven hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic signature mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states the O(n·d) time and O(k) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Signature grouping is a general technique.
