# Review summary: tide-table

Round 1, content hash 40edf4bce171. Written for a non-expert reader. One page.

## What the problem asks
A keeper logs the tide height each hour. Given the readings in order, return the position of the first hour lower than the hour before it, or report that the tide never drops.

## Why it tests real skill
The learner must compare each reading to the previous one and respect a strict "lower than" rule. Counting level stretches as drops, or starting the scan at the wrong hour, both misread the table.

## What the ghost shows
One ghost reports the hour before the actual drop; another treats a level reading as a drop. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. One review fix added the indexing concept tag a ghost legitimately uses.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the position base and the never-drop result are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a single-scan reference fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
