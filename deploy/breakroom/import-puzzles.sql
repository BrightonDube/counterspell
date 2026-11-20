-- Imports up to 3000 rated Lichess puzzles into `puzzles`.
-- The CSV never comes from the network: pipe a decompressed CC0
-- lichess_db_puzzle.csv.zst on stdin and pass the file date.
--
--   zstd -d lichess_db_puzzle.csv.zst -c | \
--     psql "$DATABASE_URL" -v source_date=2026-11-01 -f deploy/breakroom/import-puzzles.sql
--
BEGIN;
CREATE TEMP TABLE puzzle_stage (
    "PuzzleId"   text,
    "FEN"        text,
    "Moves"      text,
    "Rating"     int,
    "Deviation"  int,
    "Popularity" int,
    "NbPlays"    int,
    "Themes"     text,
    "GameUrl"    text,
    "OpeningTags" text
) ON COMMIT DROP;
\copy puzzle_stage FROM stdin WITH CSV HEADER
INSERT INTO puzzles (id, fen, moves, rating, themes, popularity, nb_plays, source_date)
SELECT "PuzzleId", "FEN", "Moves", "Rating", "Themes", "Popularity", "NbPlays",
       :'source_date'::date
FROM (
    SELECT * FROM puzzle_stage
    WHERE "Rating" BETWEEN 1000 AND 2200 AND "Popularity" >= 80
    ORDER BY "Popularity" DESC, "PuzzleId"
    LIMIT 3000
) picked
ON CONFLICT (id) DO NOTHING;
COMMIT;
