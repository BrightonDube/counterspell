## Idea
A comma-joined dictionary keeps the prefix query inside one plain string while still testing real trie reasoning.

## Why it is original
The lexicon wording, the empty-entry rule and the repeated-entry rule were written for this pack.

## Patterns used
Linear scan with a prefix test per entry.

## Checklist answers
The wording is my own (AU-1). The function uses `str` inputs and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dictionaries, empty prefixes, repeated entries and comma gaps (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) extra space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
