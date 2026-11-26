## Idea
A shared stem over a comma-joined lexicon is the rootward path of a trie, stated in plain arrays.

## Why it is original
The stem wording, the empty-dictionary answer and the repeated-entry rule were written for this pack.

## Patterns used
Position-by-position comparison across all entries.

## Checklist answers
The wording is my own (AU-1). The function uses a `str` input and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dictionaries, single entries, repeated entries, comma gaps and disjoint first runes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) extra space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
