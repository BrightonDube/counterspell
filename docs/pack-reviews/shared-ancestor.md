# Review summary: shared-ancestor

Round 1, content hash c1e16cde2be7ec97. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays holds two node indices; the keeper must return the deepest node that stands above both, or `-1` when either is unreachable.

## Why it tests real skill
The learner must map parents over the reachable tree, then climb one node's ancestors and walk the other up to the first shared cell. Returning the root, returning `a` itself, or indexing `vals[b]` unchecked each misses the real shared cell.

## What the ghost shows
One ghost always answers the root; one returns `a` itself; one reads `vals[b]` before checking `b` is in bounds and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement names the `-1` answer for unreachable nodes and the deepest-shared rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a parent-map plus two ancestor climbs fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
