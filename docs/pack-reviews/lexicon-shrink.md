# Review summary: lexicon-shrink

Round 1, content hash cd9aa8bf26979be3. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is counting the distinct entries that no other entry absorbs by proper prefix.

## Why it tests real skill
The learner must keep only the words that are not a proper stem of another word — deduplicating first so repeats name one word.

## What the ghost shows
One ghost never absorbs anything; one absorbs a word into an identical entry; one reads the first rune of an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the proper-prefix rule, the distinct-word count and the empty-dictionary answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a quadratic absorption check over distinct words fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
