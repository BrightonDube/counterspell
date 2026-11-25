# Review summary: gale-peaks

Round 1, content hash fb33c0da9d9e. Written for a non-expert reader. One page.

## What the problem asks
A storm watch logs a wind reading each hour; every width consecutive hours form a gust window. Return, in order, the highest reading of each window.

## Why it tests real skill
The learner must keep a monotonic queue of candidates so each window's peak stays front. Taking the first reading, keeping a stale peak, or reporting only the last window each misread the stream.

## What the ghost shows
One ghost reports each window's first reading as its peak; one keeps an out-of-window reading alive as the peak; one reports only the final window's peak.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty answer when the log is shorter than the width.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the monotonic deque is a hard step at this size.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
