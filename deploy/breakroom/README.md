# Break-room puzzle import

Imports rated chess puzzles from the public Lichess puzzle database into the
`puzzles` table. The server never reads the CSV at request time; this is a
one-time operator action.

## Source and licence

- File: `lichess_db_puzzle.csv.zst` from <https://database.lichess.org/#puzzles>
- Licence: Creative Commons CC0 (public domain dedication). Lichess publishes
  the database as free data for any use; the puzzles carry no copyright claim.
- Header row: `PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes,GameUrl,OpeningTags`
- Compressed with zstd; decompress with `zstd -d lichess_db_puzzle.csv.zst -c`.

## Run

```sh
zstd -d lichess_db_puzzle.csv.zst -c | \
  psql "$DATABASE_URL" -v source_date=YYYY-MM-DD -f deploy/breakroom/import-puzzles.sql
```

`source_date` is the Lichess file date you downloaded; it is stored on every
row so a later re-import from a newer file is auditable. Re-running is safe:
`ON CONFLICT (id) DO NOTHING` skips puzzles already imported.

The filter keeps rating 1000-2200 and popularity >= 80, picks the 3000 most
popular puzzles (stable order on PuzzleId), and ignores the rest. Nothing is
downloaded by the script itself.

## Test fixture

`crates/server/tests/fixtures/breakroom-sample.csv` holds 12 hand-written
synthetic rows in the same format for the import integration test. They are
not real Lichess puzzles.

## Cut rehearsal note

The break room ships behind its isolation map and was deliberately cut and
rebuilt on a scratch branch during T19; the status recorded for the release
checklist is: shipped.
