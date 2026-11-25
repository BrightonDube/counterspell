# The Equal Embers

## Task
A hearth holds ember marks in a row, each a `1` or a `2`. A span is *balanced* when it holds as many ones as twos. Call `balanced_span(embers)` and return the length of the longest balanced span, as an integer. An empty hearth holds no balanced span and returns `0`.

## Input
A list of ember marks, each `1` or `2`.

## Output
Return the length of the longest contiguous span with equal ones and twos, as an integer.

## Examples
`balanced_span([1, 2, 1, 1, 2])` returns `4`.
The last four marks hold two ones and two twos; the whole row is unbalanced because a third one sits at the start.

## Constraints
The list has at most 40 marks. Each mark is `1` or `2`.
