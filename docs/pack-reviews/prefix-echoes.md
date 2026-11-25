# Review summary: prefix-echoes

Round 1, content hash 0a95080b714c. Written for a non-expert reader. One page.

## What the problem asks
A gatekeeper's log lists marks, and each question names a stretch and a mark. For every question in order, return how many times that mark appears among the first so-many marks of the log. A stretch beyond the log counts the whole log.

## Why it tests real skill
The learner must bound each count to the named prefix and answer one count per question. Rescanning the whole log, or stopping one entry short of the stretch, both answer a different question than asked.

## What the ghost shows
One ghost counts the mark across the whole log, ignoring the stretch; another drops the last mark of the stretch. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement now says in words that an empty log answers zero.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the stretch rule and the empty-log edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, bounded per-question counting fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
