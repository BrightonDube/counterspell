# Review summary: quiet-hours

Round 1, content hash 2da6e04d55be. Written for a non-expert reader. One page.

## What the problem asks
A watch log lists a bustle reading per hour; a stretch of three consecutive hours is quiet when their readings add to no more than the quota. Count how many three-hour stretches are quiet.

## Why it tests real skill
The learner must slide a three-wide window and respect the inclusive quota. A strict-less-than comparison, counting partial edges, or sliding the wrong width each miscount boundary stretches.

## What the ghost shows
One ghost demands strictly less than the quota and loses exact fits; one counts stretches of only two readings at the log's tail; one slides a two-wide window instead of three.

## What could still be wrong
No unresolved pack issue was raised. The statement fixes the stretch at three hours and the quota as at most.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a fixed-width slide with one comparison fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
