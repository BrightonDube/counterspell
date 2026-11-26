# Review summary: nth-tail

Round 1, content hash 1ca171566bb4fac5. Written for a non-expert reader. One page.

## What the problem asks
A lantern chain holds marks; the keeper must return the mark n places before the tail, or -1 when the chain is too short.

## Why it tests real skill
The learner must walk a lead pointer n steps ahead, then walk both until the lead ends — index arithmetic cannot see the real tail. Reading marks[len-1-n], counting the lead from one, or reading the last cell each misses the contract.

## What the ghost shows
One ghost uses index arithmetic on a woven chain; one counts the lead from one; one reads the array's last cell as the tail.

## What could still be wrong
No unresolved pack issue was raised. Out-of-range distances and the empty chain are named.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a two-pointer lead/trail scheme fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
