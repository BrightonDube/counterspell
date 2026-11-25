# Review summary: peak-torch

Round 1, content hash 342c28fe5f1124. Written for a non-expert reader. One page.

## What the problem asks
Torch marks rise to a single summit then fall; the keeper must name the summit's position.

## Why it tests real skill
The learner must compare a torch with its right neighbour and keep the rising side. Answering the mark itself, reporting the first rise, or peeking past the left edge each fails.

## What the ghost shows
One ghost answers the summit's mark; one reports the first rising step; one peeks right of position zero and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement guarantees a single summit.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one neighbour-compare halving search fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
