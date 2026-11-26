# Review summary: ember-floor

Round 1, content hash 8cd76c8dc69f24cf. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks and a pick count reach the keeper; the task is the total of the k marks that sort first.

## Why it tests real skill
The learner must order the pile, cap the pick at the pile's size and answer zero on an empty pile or a zero pick.

## What the ghost shows
One ghost orders the pile the wrong way and sums the largest marks; one sums the first k entries in arrival order; one indexes an empty pile.

## What could still be wrong
No unresolved pack issue was raised. The statement names the capped pick, the repeat rule and the empty answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a bounded pick over a small pile fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
