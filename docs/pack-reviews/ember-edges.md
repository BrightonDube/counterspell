# Review summary: ember-edges

Round 1, content hash cce30d70f08567. Written for a non-expert reader. One page.

## What the problem asks
A wall holds ember marks in order; the keeper must report the first and last positions of a mark, or [-1, -1] when it is absent.

## Why it tests real skill
The learner must run two bound searches and pair them. Reporting the same edge twice, answering empty on a miss, or skipping the equality probes each breaks the pair.

## What the ghost shows
One ghost reports the first position for both edges; one answers [] instead of the pair; one never checks for equality.

## What could still be wrong
No unresolved pack issue was raised. The statement names the [-1, -1] miss answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, two bound searches merged into one pair fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
