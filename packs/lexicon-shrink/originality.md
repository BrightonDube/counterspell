## Idea
Absorption by proper-prefix keeps only the trie's word-ending nodes that have no word below them.

## Why it is original
The shrink wording, the distinct-word count and the repeated-entry rule were written for this pack.

## Patterns used
Quadratic check of each distinct word against every other.

## Checklist answers
The wording is my own (AU-1). The function uses a `str` input and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dictionaries, chains, disjoint words, identical entries and comma gaps (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(k^2) comparisons over at most k distinct words (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
