# The Cluster Step

## Task
A clustering rite reassigned every mark to its nearest hearth — tie going to the lower hearth number. Given where each mark should land and the labels it carried before, call `cluster_step(points, centroids, labels)` and return how many marks would move.

## Input
`points` are mark positions, `centroids` the hearth positions, `labels` the previous hearth index per mark (`-1` for unassigned, short for the rest).

## Output
The count of marks whose nearest hearth differs from their previous label.

## Examples
- `cluster_step([0, 10, 20], [0, 20], [0, 0, 1])` returns `0` — `10` ties and the low hearth `0` keeps it.
- `cluster_step([0, 10, 20], [0, 20], [1, 1, 1])` returns `2`.
- `cluster_step([5], [5], [0])` returns `0`.

## Constraints
- `0 <= len(points) <= 24`, `0 <= len(centroids) <= 8`, `0 <= len(labels) <= 24`, `-50 <= values <= 50`
