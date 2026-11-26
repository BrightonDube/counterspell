## Idea
Order-insensitive mix counting — the kind-outer loop keeps each mix in canonical order.

## Why it is original
The lit-combo wording, the order-collapse rule and the impossible cases were written for this pack.

## Patterns used
Coin-outer table so permutations collapse.

## Checklist answers
The wording is my own (AU-1). The function takes a `list[int]` and an `int`, and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover zero tolls, unreachable tolls, single kinds and multi-mix targets (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(kinds × target) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
