# Review summary: ember-diameter

Round 1, content hash d0f0cca234414d88. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays has a diameter; the keeper must return the edge count of the longest node-to-node walk.

## Why it tests real skill
The learner must measure the longest span at every node — through each node or inside a bough — since the answer need not pass the root. Measuring only the root span, doubling the height, or counting nodes each misses the true span.

## What the ghost shows
One ghost measures only the span through the root; one doubles the tree height; one returns the node count.

## What could still be wrong
No unresolved pack issue was raised. The statement names the edge-count answer and the empty-tree answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a per-node span computation over heights fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
