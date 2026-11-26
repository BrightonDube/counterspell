# Review summary: stale-tally

Round 1, content hash 2ff56b1b71b705dd. Written for a non-expert reader. One page.

## What the problem asks
Two version lists reach the keeper — the ledger's current versions and the ones each queued event saw; the task is the count of seats that disagree.

## Why it tests real skill
The learner must compare each pair in seat order and count every mismatch, including a fully drifted ledger.

## What the ghost shows
One ghost counts the fresh seats instead; one returns the ledger size regardless; one reads seat zero of an empty ledger.

## What could still be wrong
No unresolved pack issue was raised. The statement names the same-length contract and the stale rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a zipped mismatch count fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
