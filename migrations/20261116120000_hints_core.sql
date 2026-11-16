-- Hints core (hints spec H10): one append-only table for authored ladder reveals, AI hints and the
-- familiar's speech bubbles. `problem_hints` (the authored text) already exists in catalogue_core.
-- The server role may only read and insert here. Nothing is granted to the judge role.
-- Every statement is idempotent, so the migration can run twice.

CREATE TABLE IF NOT EXISTS hints (
    id             bigserial PRIMARY KEY,
    user_id        uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    submission_id  uuid NOT NULL REFERENCES submissions (id) ON DELETE CASCADE,
    rung           smallint CHECK (rung IS NULL OR rung BETWEEN 1 AND 4),
    source         text NOT NULL CHECK (source IN ('ladder', 'ai', 'familiar')),
    provider       text CHECK (provider IS NULL OR provider IN ('openai', 'groq')),
    model          text CHECK (model IS NULL OR char_length(model) <= 80),
    body           text NOT NULL CHECK (char_length(body) BETWEEN 1 AND 2000),
    tokens_in      int CHECK (tokens_in IS NULL OR tokens_in >= 0),
    tokens_out     int CHECK (tokens_out IS NULL OR tokens_out >= 0),
    cost_usd       numeric(10,6) NOT NULL DEFAULT 0 CHECK (cost_usd >= 0),
    cost_estimated boolean NOT NULL DEFAULT false,
    cache_key      bytea CHECK (cache_key IS NULL OR octet_length(cache_key) = 32),
    created_at     timestamptz NOT NULL DEFAULT now(),
    -- A ladder row has a rung and no provider. An AI row has a rung, a provider and a cache key.
    -- A familiar row has a provider and a cache key, and never uses the rung column.
    CHECK ( (source = 'ladder'   AND rung IS NOT NULL AND provider IS NULL AND cache_key IS NULL)
         OR (source = 'ai'       AND rung IS NOT NULL AND provider IS NOT NULL AND cache_key IS NOT NULL)
         OR (source = 'familiar' AND rung IS NULL     AND provider IS NOT NULL AND cache_key IS NOT NULL) )
);

-- One ladder reveal per rung per submission, one AI hint per rung per submission, and one familiar
-- bubble per submission. The unique indexes make concurrent reveals store a single row.
CREATE UNIQUE INDEX IF NOT EXISTS hints_one_ladder
    ON hints (submission_id, rung) WHERE source = 'ladder';
CREATE UNIQUE INDEX IF NOT EXISTS hints_one_ai
    ON hints (submission_id, rung) WHERE source = 'ai';
CREATE UNIQUE INDEX IF NOT EXISTS hints_one_familiar
    ON hints (submission_id) WHERE source = 'familiar';
CREATE INDEX IF NOT EXISTS hints_cache_key ON hints (cache_key);
CREATE INDEX IF NOT EXISTS hints_user_day ON hints (user_id, created_at) WHERE source <> 'ladder';

REVOKE ALL ON hints FROM PUBLIC;
GRANT SELECT, INSERT ON hints TO counterspell_server;
GRANT USAGE ON SEQUENCE hints_id_seq TO counterspell_server;
