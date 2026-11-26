# Review summary: lit-order

Round 1, content hash b633250ab8a3926e. Written for a non-expert reader. One page.

## What the problem asks
A chained row holds marks; the keeper must list them in the order the walk visits them.

## Why it tests real skill
The learner must collect marks by following pointers, not by reading the array in order. Returning the array order, only the head, or a reversed array each misses the woven order.

## What the ghost shows
One ghost returns the array as given; one returns only the head's mark; one reverses the raw array.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty-chain answer and the stray-cell rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a collecting walk over woven pointers fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
