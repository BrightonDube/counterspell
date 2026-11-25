# Review summary: next-lantern

Round 1, content hash b1291903e12c. Written for a non-expert reader. One page.

## What the problem asks
Each lantern in a row wants the distance to the nearest strictly taller lantern ahead of it, or zero when none stands taller.

## Why it tests real skill
The learner must keep a rising stack and respect strictness. Equal height must not count; the weaker direction, the furthest view, and at-least-as-tall each distort the distances.

## What the ghost shows
One ghost looks for a weaker lantern and answers the mirror question; one counts equal height as taller and shortens distances; one looks past the nearest taller to a further one.

## What could still be wrong
No unresolved pack issue was raised. The statement spells out that equal height does not count.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the monotonic stack is a medium idea even at this size.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
