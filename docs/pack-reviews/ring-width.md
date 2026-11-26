# Review summary: ring-width

Round 1, content hash 417f5f8daa8b3aa8. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays forms rings by depth; the keeper must return the count of nodes on the widest ring.

## Why it tests real skill
The learner must tally nodes per depth over the reachable tree and take the largest ring. Returning the deepest ring's count, the tree height, or the node count each misses the widest level.

## What the ghost shows
One ghost returns the deepest ring's count; one returns the tree height; one returns the array's length.

## What could still be wrong
No unresolved pack issue was raised. The statement defines rings by depth and names the empty-tree answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a breadth tally over depth rings fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
