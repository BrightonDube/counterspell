# Review summary: rune-depth

Round 1, content hash fb7af41880c2. Written for a non-expert reader. One page.

## What the problem asks
A directed order list and a node count reach the keeper; the task is the length of the longest forced chain, with zero when the orders cycle or a self-order exists.

## Why it tests real skill
The learner must walk the order web depth-first with a memo and a three-state cycle guard; without the guard a ring reads as a chain, and counting arcs instead of depth fails on the diamond.

## What the ghost shows
One ghost returns the arc count; one follows the orders without a cycle guard and counts the ring as a chain; one reads the first order of an empty list. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
