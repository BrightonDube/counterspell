# Review summary: cycle-ward

Round 1, content hash c0ce5c39fffb. Written for a non-expert reader. One page.

## What the problem asks
An arc list and a node count reach the keeper; the task is whether a walk can leave a node and return without reusing the arc it just crossed, where a self-loop and a doubled arc both ring.

## Why it tests real skill
The learner must union the arc ends and treat a shared root as a ring; dropping repeated arcs before checking makes the doubled arc look like a forest.

## What the ghost shows
One ghost flags any node holding two arcs as a ring; one deduplicates the arc list first so the doubled arc never rings; one reads the first arc of an empty web. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
