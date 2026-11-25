## Idea
Granary rows hold sacks. A feast pairs each sack with sacks from other rows. Count the distinct cross-row sack pairs; the tally grows large quickly.

## Why it is original
The granary-and-feast framing, the cross-row pairing rule and the wording are new for this pack.

## Patterns used
A running suffix sum so each row contributes its product with later rows once.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; counts are non-negative (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Twelve hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic pairing mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Pairwise products are a general technique.
