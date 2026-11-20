-- break-room: imported chess puzzles and cosmetic game scores.
CREATE TABLE puzzles (
    id          text PRIMARY KEY,
    fen         text NOT NULL,
    moves       text NOT NULL,
    rating      int  NOT NULL,
    themes      text NOT NULL,
    popularity  int  NOT NULL,
    nb_plays    int  NOT NULL,
    source_date date NOT NULL,
    imported_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX puzzles_rating_idx ON puzzles (rating);

CREATE TABLE game_scores (
    id         bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game       text NOT NULL CHECK (game IN ('chess_puzzle', 'chess_bot', 'brain')),
    score      int  NOT NULL CHECK (score BETWEEN 0 AND 10000),
    meta       jsonb NOT NULL CHECK (octet_length(meta::text) <= 1024),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX game_scores_user_game_created_idx
    ON game_scores (user_id, game, created_at DESC);

-- At most one chess-puzzle score per user per SAST date (the date lives in meta).
CREATE UNIQUE INDEX game_scores_one_puzzle_per_day_idx
    ON game_scores (user_id, (meta->>'date'))
    WHERE game = 'chess_puzzle';
