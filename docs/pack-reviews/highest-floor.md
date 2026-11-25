# Review summary: highest-floor

Round 1, content hash cc1f6ac4216759. Written for a non-expert reader. One page.

## What the problem asks
Floor marks rise along a wall; the keeper must name the highest mark at or below a traveller's head, or -1 when every mark is too high.

## Why it tests real skill
The learner must keep the last position that still passes. Dropping the equal-mid match, demanding a strict pass, or answering the first position each loses the boundary.

## What the ghost shows
One ghost discards the equal probe; one requires a mark strictly under the traveller; one answers the first position.

## What could still be wrong
No unresolved pack issue was raised. The statement names the -1 answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one upper-bound search returning the mark fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
