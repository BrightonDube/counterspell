# Review summary: meld-cost

Round 1, content hash bd1abd83c402b6ea. Written for a non-expert reader. One page.

## What the problem asks
Pile sizes reach the keeper; melding two piles costs their sum, and the task is the least total cost to fuse all piles into one.

## Why it tests real skill
The learner must keep the sizes in a min-heap and fuse the two smallest each step, counting the fused pile in later costs.

## What the ghost shows
One ghost fuses the two largest each step; one melds in arrival order without a heap; one pops a second pile that does not exist.

## What could still be wrong
No unresolved pack issue was raised. The statement names the cost rule and the single-pile answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a heap-driven greedy fuse fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
