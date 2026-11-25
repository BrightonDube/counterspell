# Review summary: bead-run

Round 1, content hash 1dd1e29c5dea. Written for a non-expert reader. One page.

## What the problem asks
A strand is threaded with a and b beads; a run is a stretch where the same bead repeats with no break. Return the length of the longest run.

## Why it tests real skill
The learner must track the live run and the best so far, resetting on every switch. Counting the most frequent bead, keeping only the final run, or counting groups instead of the longest each answer a different question.

## What the ghost shows
One ghost reports the most common bead's total, not the longest run; one keeps only the closing run; one counts how many runs exist instead of their length.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty strand answer of 0.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one running counter with a reset fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
