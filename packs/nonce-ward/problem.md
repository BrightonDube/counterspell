# The Nonce Ward

## Task
Each sender's seal carries a nonce that must always climb — a message whose nonce is not higher than that sender's highest seen is a replay and the ward must count it. Call `nonce_ward(msgs)` and return how many messages are replays.

## Input
`msgs` lists `[sender, nonce]` rows in delivery order; senders and nonces run `0` to `500`.

## Output
The count of rows whose nonce is at or below the highest nonce already delivered from that sender.

## Examples
- `nonce_ward([[0, 1], [0, 2], [0, 2]])` returns `1` — the repeated `2` is a replay.
- `nonce_ward([[0, 1], [1, 1], [0, 2]])` returns `0` — different senders hold different counters.
- `nonce_ward([[0, 3], [0, 1]])` returns `1` — a nonce below the high-water mark replays.

## Constraints
- `0 <= len(msgs) <= 40` rows of `[sender, nonce]` with `0 <= sender, nonce <= 500`
