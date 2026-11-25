## Idea
A gatekeeper keeps a log of marks and asks questions about only the first stretch of it. Answer each question in order: how many times a chosen mark appears among the first entries.

## Why it is original
The gatekeeper's prefix questions, the paired query rows and the wording are new for this pack.

## Patterns used
Reading a bounded prefix per query and counting matches; optionally a running tally table.

## Checklist answers
The wording is my own (AU-1). The function uses `list[int]`, `list[list[int]]` and `list[int]` return only (AU-2). Constraints match the schema; values are non-negative (AU-3). The worked example calls the function with sample 1 and explains it briefly (AU-4). Eleven hidden cases were derived by hand (AU-5, AU-6). Two ghosts are realistic prefix mistakes with hand-traced `got` values (AU-7). Four hint rungs follow the ladder (AU-8, AU-9). The complexity note states the O(q·n) time and O(q) answer space (AU-10). Reviewer items were read and met (AU-11).

## Third-party text avoided
No third-party problem statement was opened or quoted while drafting. Prefix counting is a general technique.
