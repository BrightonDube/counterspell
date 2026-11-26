# Review summary: drain-order

Round 1, content hash b6e9e2525d475086. Written for a non-expert reader. One page.

## What the problem asks
Queued events arrive as flat urgency,id pairs; the task is the id order in which a priority queue drains them.

## Why it tests real skill
The learner must decode the pairs and order by urgency first and id second — arrival order plays no part.

## What the ghost shows
One ghost drains in arrival order; one reverses the comparator and drains the least urgent first; one reads a missing id slot on an odd-length input.

## What could still be wrong
No unresolved pack issue was raised. The statement names the pair encoding and the id tie-break.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, pairing two keys and ordering them fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
