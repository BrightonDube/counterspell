# Review summary: signal-reach

Round 1, content hash bec5248a92a7. Written for a non-expert reader. One page.

## What the problem asks
An arc list and a node count reach the keeper; a signal enters at node 0 and travels every arc both ways, and the task is how many nodes it can stand on, node 0 included.

## Why it tests real skill
The learner must flood the web through both arc directions while keeping a visited ledger; counting named endpoints instead of reached nodes fails as soon as the web splits.

## What the ghost shows
One ghost returns the arc count rather than the reached nodes; one counts every node an arc names, isles unseen; one reads the first arc of an empty web. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
