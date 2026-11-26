## Idea
Exact membership over a comma-joined lexicon separates real words from prefix-only lookalikes.

## Why it is original
The ward wording, the empty-entry rule and the empty-word rule were written for this pack.

## Patterns used
Linear scan with an exact-match test per entry.

## Checklist answers
The wording is my own (AU-1). The function uses `str` inputs and returns `int` (AU-2). Constraints match the schema including the character set (AU-3). The prefix-only example is one line (AU-4). Hidden cases cover empty dictionaries, repeated entries, comma gaps and prefix-only probes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n) time and O(1) extra space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
