# Mirror Signal

## Task
At the Mirror Gate, two wardens read opposite ends of a strip of runes. They need to know how many paired runes disagree.

Write `mirror_signal(signal)` to return the number of mismatched pairs as an integer. Pair the first rune with the last, the second with the second-to-last, and continue inward. Count each mismatched pair once and ignore any unpaired middle rune.

## Input
A string `signal` containing the lowercase rune letters `k`, `m` and `v`.

## Output
Return the number of pairs whose runes differ. A signal with fewer than two runes has no pairs, so return zero.

## Examples
`mirror_signal("kmmmv")` returns `1`.

The outer pair has different runes. The next pair matches.
The middle rune has no partner, so only the outer pair counts.

## Constraints
The signal is between 0 and 48 runes long, inclusive. Every rune is `k`, `m` or `v`.
