# Review summary: sigil-board

Round 1, content hash 2fd0158539f59b15. Written for a non-expert reader. One page.

## What the problem asks
A square board of n rows and columns must hold n sigils so none share a row, column or diagonal; the task is the number of distinct placements.

## Why it tests real skill
The learner must place row by row and prune columns plus both diagonals; single-direction checks leak clashes and column-only checks return permutations.

## What the ghost shows
One ghost prunes only shared columns and counts every row order; one checks a single diagonal direction; one reads past a hard-coded answer table on a large board.

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
