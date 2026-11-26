## Idea
Two-way splits grow into `ways` unlabelled heaps — the search assigns marks to heap totals and must dedupe symmetric group orderings.

## Why it is original
The unlabelled-heaps wording, the ways bound and the dedupe requirement were written for this pack.

## Patterns used
Group-assignment backtracking with a fill-order canonicalisation: the next mark may only join a used heap or the first empty one.

## Checklist answers
The wording is my own (AU-1). The function uses an `int_array` and an `int` input and returns `int` (AU-2). Constraints match the schema (AU-3). The examples stay one line each (AU-4). Hidden cases cover impossible splits, single marks, zero marks, three ways and the bounds (AU-5, AU-6). Three ghosts carry hand-traced values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states the bounded ways^n assignment walk (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting.
