## Idea
Word segmentation over a lexicon is the trie's deepest walk: each cut must land on a real word boundary.

## Why it is original
The ward wording, the reusable-entry rule and the empty-string answer were written for this pack.

## Patterns used
Position-by-position reachability over entry matches.

## Checklist answers
The wording is my own (AU-1). The function uses `str` inputs and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The examples stay one line each (AU-4). Hidden cases cover empty strings, empty dictionaries, overlaps, backtracking cuts and repeated use (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n * w) reachability over at most w entries (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
