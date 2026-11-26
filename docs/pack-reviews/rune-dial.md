# Review summary: rune-dial

Round 1, content hash c0d9cba8a09cb809. Written for a non-expert reader. One page.

## What the problem asks
Marks land one by one on a dial; the task is, after each mark, twice the running median — twice the middle on odd counts, the two middles' sum on even counts.

## Why it tests real skill
The learner must split the seen marks across a low max-heap and a high min-heap, rebalance after each mark and read the tops.

## What the ghost shows
One ghost doubles only the lower middle on even counts; one reads the middle of each prefix in arrival order; one reads the median before the first mark lands.

## What could still be wrong
No unresolved pack issue was raised. The statement names the doubled output and the empty answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, twin-heap rebalancing across every prefix fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
