## Idea
Counting topological orders turns the dependency check into a search over free nodes.

## Why it is original
The line-up wording, the repeat rule and the self-order clause were written for this pack.

## Patterns used
Kahn-style removal counting every extension of the line-up.

## Checklist answers
The wording is my own (AU-1). The function uses a `list[list[int]]` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover cycles, self-orders, repeats, lone nodes and full webs (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states a bounded O(n × n!) walk within the bound (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
