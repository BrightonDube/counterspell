# Prefix Echoes

## Task
The gatekeeper's log lists mark after mark. Each question names a stretch `k` and a mark `v`. Call `prefix_counts(marks, queries)`. For each question in order, return how many times `v` appears among the first `k` marks of the log. A stretch longer than the log counts the whole log. An empty log answers `0` to every question.

## Input
A list of marks, then a list of questions. Each question is a pair `[k, v]`. `k` counts marks from the start.

## Output
Return a list of counts as integers, one per question, in the same order.

## Examples
`prefix_counts([4, 4, 2, 4], [[2, 4], [4, 2]])` returns `[2, 1]`.
Among the first `2` marks, `4` appears twice. Among the first `4` marks, `2` appears once.

## Constraints
The log has at most 40 marks, each from `0` to `40`. There are at most 20 questions. Each `k` is from `0` to `40`; each `v` is from `0` to `40`.
