# Review summary: harvest-quota

Round 1, content hash ff9e12a3162f. Written for a non-expert reader. One page.

## What the problem asks
A picker may keep any unbroken run of marked vines while the marks add to no more than the quota. Return the length of the longest keepable run.

## Why it tests real skill
The learner must shrink from the left while the running sum tops the quota. A strict bound, prefix-only runs, or counting qualifying windows instead of the longest each under- or over-answer.

## What the ghost shows
One ghost demands strictly less than the quota and drops exact fits; one tries only prefixes; one counts how many windows fit instead of returning the longest.

## What could still be wrong
No unresolved pack issue was raised. The statement fixes marks at 1 to 9 so the two-pointer invariant holds.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the sum-bounded two-pointer walk fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
