# Review summary: reorder-embers

Round 1, content hash fe791e8fbd4aa784. Written for a non-expert reader. One page.

## What the problem asks
A lantern chain holds marks; the keeper must weave them — first half in order, second half reversed, interleaved.

## Why it tests real skill
The learner must collect the walk, split it, reverse the back half and interleave — odd counts keep the middle mark last-paired. Interleaving without the flip, appending the reversed half, or splitting odd counts unevenly each misses the weave.

## What the ghost shows
One ghost interleaves without reversing the back half; one appends the reversed half instead of interleaving; one drops the middle mark on odd counts.

## What could still be wrong
No unresolved pack issue was raised. The empty chain and odd-length rule are named.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, collect, split, reverse and interleave fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
