## Idea
A hearth holds ember marks of two kinds, one and two. A span is balanced when it holds as many ones as twos. Find the length of the longest balanced span.

## Why it is original
The twin-ember framing, the two-symbol balance rule and the wording are new for this pack.

## Patterns used
Folding two symbols into one running balance and keeping the first position each balance appeared.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]` and `int` only (AU-2). Constraints match the schema; marks are restricted to one or two (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Twelve hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic balance mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(n) space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Running balances are a general technique.
