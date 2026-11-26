# Review summary: shared-stem

Round 1, content hash ee4f3ed21a41e710. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is finding the longest rune run every dictionary entry shares at the front.

## Why it tests real skill
The learner must shrink a candidate stem until every entry accepts it — comparing only a pair or trusting the first entry's length both miss the shared run.

## What the ghost shows
One ghost compares only the first pair of entries; one returns the first entry's whole length; one reads the first rune of an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the shared-run rule, the single-entry answer and the comma-gap behaviour.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a shrinking-stem scan across entries fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
