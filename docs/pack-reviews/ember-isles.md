# Review summary: ember-isles

Round 1, content hash fe86411705e8. Written for a non-expert reader. One page.

## What the problem asks
An arc list and a node count reach the keeper; the web splits into isles joined by arcs, and the task is the number of isles with each lone node counting as its own.

## Why it tests real skill
The learner must restart the flood at every unvisited node; counting only nodes that arcs name loses every lone node.

## What the ghost shows
One ghost counts only the isles that arcs create and drops lone nodes; one takes the arc count for the isle count; one indexes the first node's list on an empty web. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
