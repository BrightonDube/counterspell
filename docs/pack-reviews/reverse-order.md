# Review summary: reverse-order

Round 1, content hash 74654cd1a30cf5a4. Written for a non-expert reader. One page.

## What the problem asks
A chained row holds marks; the keeper must list them tail first, in reverse chain order.

## Why it tests real skill
The learner must collect the walk then flip it — pointer order, not array order, both ways. Returning the walk order, reversing the raw array, or returning the head only each misses the contract.

## What the ghost shows
One ghost forgets the flip; one reverses the raw array instead of the chain order; one returns only the head's mark.

## What could still be wrong
No unresolved pack issue was raised. The statement names stray cells and the empty chain.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a collect-plus-flip over woven pointers fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
