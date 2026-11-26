# Review summary: order-count

Round 1, content hash 07185ed8e389. Written for a non-expert reader. One page.

## What the problem asks
A directed order list and a node count reach the keeper; the task is the number of line-ups that honour every order, with zero when the orders cycle.

## Why it tests real skill
The learner must free a node only once every order into it is placed and count each way to finish; returning the factorial or a single greedy line-up both miss the count.

## What the ghost shows
One ghost returns n factorial without reading the orders; one counts a single line-up where the rules hold; one pops a free node that does not exist once the rules cycle. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
