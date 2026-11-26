## Idea
Autocomplete fan-out over a lexicon is the direct read of a trie node's children, stated over comma-joined text.

## Why it is original
The rune wording, the exact-entry rule and the distinct-rune count were written for this pack.

## Patterns used
Linear scan collecting a set of next-runemarks.

## Checklist answers
The wording is my own (AU-1). The function uses `str` inputs and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty dictionaries, exact entries, repeated completions and comma gaps (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(k) space for the rune set (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
