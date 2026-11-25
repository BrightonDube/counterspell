# Review summary: warden-sums

Round 1, content hash 8a8a0967afcc. Written for a non-expert reader. One page.

## What the problem asks
A warden chalks integer readings that may be negative. A run of days is ritual when its readings sum to the quota exactly. Given the readings and the quota, return how many contiguous runs are ritual.

## Why it tests real skill
The learner must count subarray sums under negative values. A running prefix map finds each run in one pass; scanning from the first day only, or abandoning a run once it passes the quota, both miss ritual runs.

## What the ghost shows
One ghost counts only runs that begin on the first day; another abandons a run once it passes the quota, though later readings can pull it back. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. One review fix added the edge-cases concept tag a ghost legitimately uses.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the signed readings, the quota and the empty edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the negative-aware prefix map fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
