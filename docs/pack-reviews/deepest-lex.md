# Review summary: deepest-lex

Round 1, content hash fdfbd39643244c38. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is returning the length of the longest dictionary entry.

## Why it tests real skill
The learner must measure each entry, keep the largest mark and treat comma gaps as nothing.

## What the ghost shows
One ghost returns the last entry's length; one returns the whole string length including commas; one takes max() of an empty dictionary.

## What could still be wrong
No unresolved pack issue was raised. The statement names the length rule, the tie behaviour and the empty-dictionary answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a maximum scan over entry lengths fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
