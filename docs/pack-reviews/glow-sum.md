# Review summary: glow-sum

Round 1, content hash 1c60f6527bf822d0. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays holds marks; the keeper must add the marks of every node the walk reaches.

## Why it tests real skill
The learner must accumulate over the reachable nodes only. Summing every array cell, summing only leaves, or summing only the right bough each leaves marks in or out wrongly.

## What the ghost shows
One ghost sums every cell including strays; one sums only leaf marks; one walks only the right bough.

## What could still be wrong
No unresolved pack issue was raised. The statement names the reachable-only rule and the empty-tree answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a walk that accumulates marks fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
