# Review summary: warm-window

Round 1, content hash 4b6463e07e38. Written for a non-expert reader. One page.

## What the problem asks
A keeper logs warm readings one by one; a hearth may sit under any width consecutive readings and its glow is their sum. Return the largest glow any placement can reach, or 0 when fewer than width readings exist.

## Why it tests real skill
The learner must slide a running sum instead of re-adding every window or trusting one position. Keeping only the first window, dropping the tail window, or answering the single largest reading each miss the real maximum.

## What the ghost shows
One ghost reports only the first window's glow; one forgets the last window entirely; one answers the single largest reading and ignores width.

## What could still be wrong
No unresolved pack issue was raised. The statement names the 0 answer when the log is shorter than the width.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one running sum with a length guard fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
