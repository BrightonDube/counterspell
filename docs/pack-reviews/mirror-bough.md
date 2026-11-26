# Review summary: mirror-bough

Round 1, content hash 30034fa32a901b4f. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays must answer `1` when its left and right boughs mirror each other in shape and marks, `0` otherwise.

## Why it tests real skill
The learner must pair mirrored positions — left of one side with right of the other — and compare marks. Comparing same-side children, checking only the root's pair, or following an unchecked index each misses mismatches.

## What the ghost shows
One ghost pairs same-side children instead of mirrored sides; one checks only the root's two children; one follows a dangling index and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement names the mirrored pairing rule and the empty-tree answer `1`.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a paired walk over mirrored positions fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
