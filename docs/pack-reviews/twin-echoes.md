# Review summary: twin-echoes

Round 1, content hash 3a9c1ebc0828. Written for a non-expert reader. One page.

## What the problem asks
The vault walls echo markings. Given the echoes in order, return how many distinct markings appear at least twice; a marking counted many times still counts once.

## Why it tests real skill
The learner must tally markings and count those crossing a threshold. Counting echo occurrences instead of distinct markings, or demanding three echoes, both miscount the vault.

## What the ghost shows
One ghost counts every repeated occurrence rather than each repeating marking once; another asks for three echoes where the rule says two. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement now says in words that an empty list scores zero.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the distinct-marking rule and the empty edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, tally-and-threshold fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
