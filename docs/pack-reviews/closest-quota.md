# Review summary: closest-quota

Round 1, content hash 0455b4edb4b4. Written for a non-expert reader. One page.

## What the problem asks
A rite needs a quota and a sorted column of marks is weighed in pairs. Return the pair sum nearest the quota; ties return the smaller sum.

## Why it tests real skill
The learner must walk the ends toward the quota and remember the nearest sum. Weighing only the first pair misses every better pairing, and refusing sums above the quota drops the nearer answer on the far side.

## What the ghost shows
One ghost weighs only the two smallest marks; another refuses to consider a sum above the quota. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that a column with fewer than two marks returns `0`, and names the tie rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the tie rule and the short-column edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, guided contraction with a tie rule fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
