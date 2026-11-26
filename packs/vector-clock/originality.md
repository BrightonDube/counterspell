## Idea
Happens-before comparison over tally vectors — all-less means ordered, mixed means concurrent.

## Why it is original
The scribe wording, the four-way return contract and the case set were written for this pack.

## Patterns used
Two-flag sweep comparing componentwise.

## Checklist answers
The wording is my own (AU-1). The function takes two `list[int]`s and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover equality, strict order and concurrent vectors (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states O(k) (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
