# Review summary: ember-split

Round 1, content hash 4d3545f111f70d74. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks reaches the keeper; the task is the number of ways to split it into two non-empty halves whose marks total the same.

## Why it tests real skill
The learner must search subsets at half the total and halve for unorderedness; cut points and labelled sides both misread the count.

## What the ghost shows
One ghost returns the subset count unhalved and counts each split twice; one splits only at cut positions and misses non-adjacent halves; one reads the first mark of an empty pile.

## What could still be wrong
No unresolved pack issue was raised. The statement names the call, the bounds and every edge answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the bound keeps the search inside the declared difficulty.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
