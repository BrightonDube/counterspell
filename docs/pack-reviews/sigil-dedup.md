# Review summary: sigil-dedup

Round 1, content hash b4dafa49d314. Written for a non-expert reader. One page.

## What the problem asks
A warden tallies sigil marks in a column that always runs smallest first, so equal marks sit side by side. Return how many different marks the column carries.

## Why it tests real skill
The learner must compare each mark with its neighbour, not count the column. Counting every entry or counting only the places where marks change both give wrong answers on ordinary columns.

## What the ghost shows
One ghost reports the column's length; another counts only the transitions and forgets the very first mark. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that an empty column holds no marks.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the empty-column edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a short reference with a single key idea fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
