## Idea
Echo markings repeat in the vault. Count how many distinct markings appear more than once, ignoring how many extra times each one echoes.

## Why it is original
The echo-chamber framing and wording are new, and the cases were hand-built for this pack. The contract counts distinct repeated values, not repeat occurrences.

## Patterns used
A frequency table followed by a filter on counts.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; markings are non-negative (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Eleven hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic occurrence-versus-distinct and threshold mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(k) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Duplicate detection by frequency is a general technique.
