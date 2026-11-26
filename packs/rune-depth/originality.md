## Idea
Longest chain in a directed order web — the dependency-order problem framed as depth.

## Why it is original
The rune-depth wording, the cycle-is-zero rule and the repeat clause were written for this pack.

## Patterns used
Memoised depth-first walk with a three-state cycle guard.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover cycles, self-orders, repeats, fans, folds and lone nodes (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n + arcs) with the memo (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
