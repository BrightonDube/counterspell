## Idea
Counting picks is the bounded choose-two-branches search: each candle is taken or left.

## Why it is original
The altar wording, the k-over-n answer and the pick-none rule were written for this pack.

## Patterns used
Take/leave recursion, equivalent to the binomial count.

## Checklist answers
The wording is my own (AU-1). The function uses two `int` inputs and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover zero picks, all picks, k over n and the bounds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(n * k) or bounded recursion (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
