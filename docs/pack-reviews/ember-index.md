# Review summary: ember-index

Round 1, content hash cbe2a65b06ee4e. Written for a non-expert reader. One page.

## What the problem asks
A wall holds ember marks in low-to-high order; the keeper must report where a given mark sits, or -1 when it is absent.

## Why it tests real skill
The learner must halve the ordered range and return -1 honestly on a miss. Checking only the middle, returning a position after an empty wall, or answering only when the first mark matches each miss the contract.

## What the ghost shows
One ghost reports only the middle probe; one reads the first position before checking the wall is empty; one answers only when the first mark matches.

## What could still be wrong
No unresolved pack issue was raised. The statement names the -1 answer and the empty wall.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one halving loop with an absent answer fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
