# Review summary: triage-mark

Round 1, content hash f2ef777b91ae3cda. Written for a non-expert reader. One page.

## What the problem asks
A queue of urgency marks reaches the keeper; the task is the index of the most urgent seat, earliest index on ties.

## Why it tests real skill
The learner must keep the best (mark, index) pair through one scan, keep the earlier index on a tie and answer -1 on an empty queue.

## What the ghost shows
One ghost lets the latest tie win; one returns the urgency value instead of its index; one reads the head of an empty queue.

## What could still be wrong
No unresolved pack issue was raised. The statement names the tie-break and the empty answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a single best-index scan fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
