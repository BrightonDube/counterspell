# Review summary: cauldron-queue

Round 1, content hash 388d597a4e82. Written for a non-expert reader. One page.

## What the problem asks
Marks join the back of a cauldron line and zeros serve the front; count how many marks were actually served.

## Why it tests real skill
The learner must track who waits, not just tally zeros. Serving without a waiter, counting the leftovers, or serving the wrong end each break on interleaved sequences.

## What the ghost shows
One ghost reports the marks still waiting rather than those served; one counts every zero as a serving even on an empty line; one serves the front unguarded and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement says an empty line serves nothing.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the first-in, first-out walk with one guard fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
