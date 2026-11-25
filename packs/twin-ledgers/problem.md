# Twin Ledgers

## Task
Two vault ledgers list rows of `[key, amount]`. A key can appear on many rows of one ledger, and its total is the sum of its amounts there. Call `matching_totals(entries_a, entries_b)` and return how many keys have the same total in both ledgers, as an integer. A key present in only one ledger never counts. Two empty ledgers agree on nothing and return `0`.

## Input
Two lists of rows. Each row is a pair `[key, amount]`.

## Output
Return the number of keys whose totals agree in both ledgers, as an integer.

## Examples
`matching_totals([[1, 5], [2, 3]], [[1, 4], [1, 1], [3, 9]])` returns `1`.
Key `1` totals `5` in the first ledger and `4 + 1` in the second, so it agrees. Keys `2` and `3` appear in one ledger only.

## Constraints
Each ledger has at most 20 rows. Each key and amount is from `0` to `50`.
