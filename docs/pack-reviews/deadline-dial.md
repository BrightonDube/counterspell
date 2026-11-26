# Review summary: deadline-dial

Round 1, content hash 2a800f51dfee0521. Written for a non-expert reader. One page.

## What the problem asks
Jobs arrive as flat deadline,duration pairs; the task is the most jobs any schedule can finish on time.

## Why it tests real skill
The learner must take jobs in deadline order, hold durations in a max-heap and drop the longest held job whenever the dial overruns — including the one just added.

## What the ghost shows
One ghost counts every job whose duration alone fits its deadline; one walks jobs in duration order; one drops from an empty heap on the first overrun.

## What could still be wrong
No unresolved pack issue was raised. The statement names the deadline order, the keep-or-drop rule and the empty answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, deadline sorting plus heap eviction fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
