# Review summary: split-ward

Round 1, content hash fae2e01f108d747b. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is deciding whether a rune string cuts fully into dictionary entries.

## Why it tests real skill
The learner must track which positions are reachable — an entry may close many gaps and a greedy first match can strand the tail.

## What the ghost shows
One ghost takes the first matching entry and never backtracks; one burns each entry on first use; one reads the first rune of an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the reuse rule, the empty-string answer and the no-cut answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, position reachability over entries fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
