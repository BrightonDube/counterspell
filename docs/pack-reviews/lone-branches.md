# Review summary: lone-branches

Round 1, content hash ca3c50b3d56d1d11. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays has parents with exactly one child; the keeper must count them.

## Why it tests real skill
The learner must distinguish exactly-one-child parents from childless nodes and two-child parents. Counting every parent, counting only two-child nodes, or checking only `right` each miscounts.

## What the ghost shows
One ghost counts every node with any child; one counts only two-child parents; one checks only the `right` slot.

## What could still be wrong
No unresolved pack issue was raised. The statement defines the exactly-one rule and the reachable-only rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a walk with a two-way child check fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
