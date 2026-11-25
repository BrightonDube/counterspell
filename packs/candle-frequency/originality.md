## Idea
A vault ledger records each candle's burn size. Report which burn size was logged most often, and when two sizes tie, report the smaller one.

## Why it is original
The vault-tally framing, the tie rule and the wording are new. The examples were built by hand for this pack.

## Patterns used
A frequency table and a single pass to keep the best value under an explicit tie rule.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; sizes are positive so no "integer" wording is needed (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Eleven hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic value-versus-frequency and tie-order mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(k) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Frequency tables are a general technique used without borrowed wording.
