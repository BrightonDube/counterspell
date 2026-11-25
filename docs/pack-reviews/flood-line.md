# Review summary: flood-line

Round 1, content hash 3f0e45bd8b50. Written for a non-expert reader. One page.

## What the problem asks
A wall of posts holds rainwater in its dips; each gap fills to the lower of the tallest bounds on either side, minus its own height. Return the total trapped water.

## Why it tests real skill
The learner must track a running bound on each side and pour only at the shorter end. Bounding every gap by the left wall alone over-fills dips that spill right; filling the interior to the end posts ignores a taller post standing in the dip.

## What the ghost shows
One ghost bounds every gap by the tallest post to its left and forgets the right wall; another fills every interior gap to the end posts' height. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that a wall with fewer than three posts traps none.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the bound rule and the short-wall edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, two-sided running bounds fit hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
