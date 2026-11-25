# Review summary: sigil-groups

Round 1, content hash 520c9873b030. Written for a non-expert reader. One page.

## What the problem asks
Every rune marking carries a sigil, the sum of its decimal digits. Given the markings, group them by sigil and return the size of the largest group.

## Why it tests real skill
The learner must derive the grouping key per marking, tally groups, and answer a group size. Grouping by the marking itself, or answering the winning sigil instead of its group's size, both miss the join.

## What the ghost shows
One ghost groups by the marking value so shared sigils never meet; another answers the winning sigil rather than how many markings share it. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the sigil definition and the empty edge are named.
- Orig
