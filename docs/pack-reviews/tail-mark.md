# Review summary: tail-mark

Round 1, content hash c10f5001dc3d214e. Written for a non-expert reader. One page.

## What the problem asks
A chained row holds marks; the keeper must return the tail link's mark, or -1 when the chain is empty.

## Why it tests real skill
The learner must keep the last index the walk visited and read its mark. Reading the array's last cell or its first cell each misses the real tail when strays trail the chain.

## What the ghost shows
One ghost reads the last array cell; one answers the head's mark; one reads past the array end and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement names the -1 answer for an empty chain.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a walk that keeps the last visited index fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
