# Review summary: kindred-path

Round 1, content hash 96e22f6b7a9e0315. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays holds marks; the keeper must answer `1` when some root-to-leaf run of marks adds to the target, `0` otherwise.

## Why it tests real skill
The learner must carry a running total and check it only at leaves. Checking the target at every node, summing all marks once, or stopping early each misses leaf paths.

## What the ghost shows
One ghost checks the running total at every node instead of only leaves; one compares the whole-tree sum once; one stops at the first match along a partial run.

## What could still be wrong
No unresolved pack issue was raised. The statement names the leaf-only rule and the empty-tree answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a running-total walk with a leaf-only check fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
