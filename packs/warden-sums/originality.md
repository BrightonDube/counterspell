## Idea
A warden chalks daily readings that may dip below zero. A span is ritual when its readings sum exactly to the quota. Count the ritual spans.

## Why it is original
The warden-and-quota framing, the signed readings and the wording are new for this pack.

## Patterns used
Prefix sums with a tally map of earlier prefixes.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` params and `int` only (AU-2). Constraints match the schema; readings are integers that may be negative and the statement says so (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Twelve hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic prefix mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Prefix sums are a general technique.
