# Review summary: sentry-gap

Round 1, content hash 6c70f7118c70a3aa. Written for a non-expert reader. One page.

## What the problem asks
A wall of posts, a sentry count and a required gap reach the keeper; the task is the number of layouts with that many empty posts between sentries.

## Why it tests real skill
The learner must run a place-or-skip walk where each sentry forces a tunable run of empties; reading the gap as spread distance miscounts every case.

## What the ghost shows
One ghost reads the gap as one fewer empty post and lets sentries stand closer; one ignores the gap and counts every pair of posts; one divides by the sentry count when none are wanted.

## What could still be wrong
No unresolved pack issue was raised. The statement names the call, the bounds and every edge answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the bound keeps the search inside the declared difficulty.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
