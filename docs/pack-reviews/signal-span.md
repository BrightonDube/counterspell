# Review summary: signal-span

Round 1, content hash 01d5d471a53d. Written for a non-expert reader. One page.

## What the problem asks
An arc list and a node count reach the keeper; a signal leaves node 0 and the task is the greatest hop count needed to reach any node it can touch.

## Why it tests real skill
The learner must layer the web breadth-first rather than walk it deep; counting reached nodes or arcs instead of hop layers both fail on a star.

## What the ghost shows
One ghost returns the arc count; one counts reached nodes instead of hop distance; one reads the first arc of an empty web. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
