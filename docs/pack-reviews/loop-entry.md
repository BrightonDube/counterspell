# Review summary: loop-entry

Round 1, content hash e65037e4d65324f5. Written for a non-expert reader. One page.

## What the problem asks
A chain may be knotted; the keeper must return the index where the loop begins, or -1 when it ends cleanly.

## Why it tests real skill
The learner must name the first index visited twice — a flag, the last seen index, or index 0 each miss where the loop actually closes.

## What the ghost shows
One ghost returns a 1/0 flag instead of the entry; one returns the loop's last index; one reports every loop as starting at index 0.

## What could still be wrong
No unresolved pack issue was raised. Unreachable loops are named as not counting.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, finding the first re-visited index fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
