# Review summary: drift-alarm

Round 1, content hash 829f92069728. Written for a non-expert reader. One page.

## What the problem asks
A tide gauge writes a reading each hour; an alarm rings at each hour whose reading stands strictly above every reading in the previous width hours. Count the alarms.

## Why it tests real skill
The learner must compare each hour against the whole look-back window, strictly. Including the hour itself, accepting equal readings, or checking only one neighbour each ring false alarms.

## What the ghost shows
One ghost compares the hour against a window that includes itself and never rings; one accepts equal readings as alarms; one checks only the lowest past reading.

## What could still be wrong
No unresolved pack issue was raised. The statement fixes strictness and the look-back excludes the hour itself.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a window-maximum comparison at every step fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
