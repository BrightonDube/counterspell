# Lantern Ledger

## Task

An archway lantern in the Grimoire vault has a charge meter. Its ledger lists changes to that meter in the order they happened. The vault keeper makes a mark whenever the meter reads the inspection level after a change.

Call `lantern_ledger(changes, start, level)` and return the number of marks as an integer. Start the meter at `start`. For each entry in `changes`, add it to the meter, then check whether the meter equals `level`.

## Input

`changes` is a list of integers. `start` is the opening reading and `level` is the inspection level. Both are integers. The meter can move outside the input bounds as changes are added.

## Output

Return the integer number of matching readings after entries. The opening reading earns no mark. Repeated matches count separately, even when a change is zero. An empty ledger earns no marks.

## Examples

`lantern_ledger([4, -2, 2, 0, -3], 1, 5)` returns `3`.
The readings after each entry are `5`, `3`, `5`, `5` and `2`.
The first, third and fourth entries each earn a mark.

`lantern_ledger([], -2, -2)` returns `0`. The opening reading matches, but the ledger has no entries.

`lantern_ledger([-3, -2, 4, -4], 2, -3)` returns `2`. The second and fourth entries leave the meter at the inspection level.

## Constraints

The ledger has at most 32 entries. Each change, the opening reading and the inspection level are integers from -100 to 100, inclusive.
