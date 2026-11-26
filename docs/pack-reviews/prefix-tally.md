# Review summary: prefix-tally

Round 1, content hash e6fd66c4ba50d648. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is counting every dictionary entry that begins with a given stem.

## Why it tests real skill
The learner must test each entry against the stem, keep comma gaps out of the count and still count repeated entries.

## What the ghost shows
One ghost counts every entry regardless of the stem; one treats the empty stem as matching nothing; one accepts a stem found anywhere inside an entry.

## What could still be wrong
No unresolved pack issue was raised. The statement names the repeated-entry rule, the empty-stem answer and the comma-gap behaviour.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a per-entry stem check over the dictionary fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
