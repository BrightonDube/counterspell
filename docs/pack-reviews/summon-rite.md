# Review summary: summon-rite

Round 1, content hash fba3e4c567d11a02. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks and a satchel total reach the keeper; the task is whether any pick of marks adds to the total exactly.

## Why it tests real skill
The learner must walk picks and skips under a running total and stop as soon as the total tips over; a single greedy pass misses picks that only work together.

## What the ghost shows
One ghost counts every pick as a hit and returns the subset count; one marks every mark as needed and always says yes on a non-empty pile; one reads the first mark of an empty pile.

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
