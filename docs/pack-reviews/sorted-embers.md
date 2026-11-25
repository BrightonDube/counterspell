# Review summary: sorted-embers

Round 1, content hash da5d1eeb7f40. Written for a non-expert reader. One page.

## What the problem asks
Ember readings arrive ordered smallest first and some run below zero. Return every reading squared, still ordered smallest first.

## Why it tests real skill
The learner must see that the biggest square hides at an end, never the middle. Squaring each reading where it stands leaves cold-end squares at the front, and returning the largest first reads the table backwards.

## What the ghost shows
One ghost squares each reading in place and keeps the input order; another squares, sorts, and returns the result largest first. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that an empty list of readings returns an empty list.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the empty-list edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the end-magnitude insight lifts it out of easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
