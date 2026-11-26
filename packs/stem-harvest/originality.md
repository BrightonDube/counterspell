## Idea
Depth by longest leading word is prefix aggregation over the trie: each word hangs from its deepest leading word rather than from every stem above it.

## Why it is original
The harvest wording, the longest-stem rule and the distinct-word total were written for this pack.

## Patterns used
Quadratic scan keeping the deepest proper prefix per word.

## Checklist answers
The wording is my own (AU-1). The function uses a `str` input and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dictionaries, chains, branches, repeated entries and comma gaps (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(k^2) prefix tests over at most k distinct words (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
