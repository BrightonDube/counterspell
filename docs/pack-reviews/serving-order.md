# Review summary: serving-order

Round 1, content hash e9b7364161df. Written for a non-expert reader. One page.

## What the problem asks
Marks join the back of a line and zeros serve the front; return the served marks in the order they left.

## Why it tests real skill
The learner must simulate a real first-in, first-out line and record each serving. Serving the back reverses the answer; returning the waiters answers the mirror question.

## What the ghost shows
One ghost serves from the back like a stack and reverses the order; one takes the front unguarded and crashes on an empty line; one returns what still waits rather than what left.

## What could still be wrong
No unresolved pack issue was raised. The statement says the second zero on an empty line records nothing.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, recording a queue walk fits medium while the shape stays small.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
