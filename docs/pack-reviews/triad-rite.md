# Review summary: triad-rite

Round 1, content hash 8191314a4099. Written for a non-expert reader. One page.

## What the problem asks
A rite needs three candles whose marks sum to a quota. The sorted column may repeat marks, and every choice of three different candles counts as its own triad. Return the count.

## Why it tests real skill
The learner must fix one mark, then walk an opposing pair over the rest — counting whole runs when the ends share a mark. Counting each value triple once drops repeated-mark triads; counting every ordering counts one triad many times.

## What the ghost shows
One ghost skips repeated marks and counts only value triads; another counts every ordering of three positions, inflating each triad sixfold. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that fewer than three candles give none, and that same-reading marks count separately.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the position-counting rule and the short-column edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, run counting on a quadratic anchor pass fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
