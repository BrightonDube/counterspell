# Review summary: valley-low

Round 1, content hash 41155f1b987dd8. Written for a non-expert reader. One page.

## What the problem asks
A cliff edge holds distinct marks rotated from sorted order; the keeper must name the mark at the valley.

## Why it tests real skill
The learner must compare each probe with the far end and keep the half holding the break. Answering the first mark, reporting the break's position, or answering the largest mark each fails.

## What the ghost shows
One ghost answers the first mark; one reports where the break sits; one answers the largest mark.

## What could still be wrong
No unresolved pack issue was raised. The statement guarantees at least one mark.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one break-point halving search fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
