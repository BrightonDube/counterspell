# Review summary: median-ward

Round 1, content hash 377e42c2b54fc779. Written for a non-expert reader. One page.

## What the problem asks
A ward of marks reaches the keeper; the task is twice the median — twice the middle on odd counts, the two middles' sum on even counts.

## Why it tests real skill
The learner must order the ward, read one or two middles and answer zero on an empty ward.

## What the ghost shows
One ghost doubles only the lower middle on even counts; one reads the middle of the ward in arrival order; one reads the middle of an empty ward.

## What could still be wrong
No unresolved pack issue was raised. The statement names the doubled contract and the empty answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a two-way middle read with the even-count rule fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
