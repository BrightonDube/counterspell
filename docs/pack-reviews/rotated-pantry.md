# Review summary: rotated-pantry

Round 1, content hash e13b332ea959a9. Written for a non-expert reader. One page.

## What the problem asks
A pantry shelf holds distinct marks in rotated order; the keeper must find a mark's position, or -1 when it is absent.

## Why it tests real skill
The learner must decide which half stays ordered and keep the half that can hold the mark. A plain sorted-shelf search, reporting the break's position, or reading an empty shelf each fails.

## What the ghost shows
One ghost searches as if never rotated; one reports where the shelf breaks instead of the mark; one reads position zero unguarded.

## What could still be wrong
No unresolved pack issue was raised. The statement keeps marks distinct so the halves decide cleanly.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one half-discard search with an ordered-half test fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
