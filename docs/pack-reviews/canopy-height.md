# Review summary: canopy-height

Round 1, content hash 8a2fe98fcbec9fa6. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays has a height; the keeper must return the number of nodes on the longest root-to-leaf path.

## Why it tests real skill
The learner must track depth while walking and report a node count, not an edge count. Returning the array length, counting edges, or reading a dangling index each misses the real height.

## What the ghost shows
One ghost returns the array's length; one counts edges instead of nodes; one follows a wild child index and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement defines height in nodes and names the empty-tree answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a depth-tracked walk with a node-count answer fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
