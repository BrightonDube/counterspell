-- Problem catalogue core tables (problem-catalogue C01).
-- Tables and CHECKs only. Immutability and publish-gate triggers come in the next task.
-- Every statement is idempotent (CREATE TABLE IF NOT EXISTS) so the migration can run twice.

CREATE TABLE IF NOT EXISTS tracks (
    slug        text PRIMARY KEY CHECK (slug ~ '^[a-z][a-z0-9-]{1,39}$'),
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    sort_order  int  NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS concepts (
    id          text PRIMARY KEY CHECK (id ~ '^[a-z][a-z0-9-]{1,39}$'),
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    sort_order  int  NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS problems (
    slug              text PRIMARY KEY CHECK (slug ~ '^[a-z][a-z0-9-]{2,39}$'),
    title             text NOT NULL CHECK (char_length(title) BETWEEN 1 AND 80),
    track_slug        text NOT NULL REFERENCES tracks (slug) ON DELETE RESTRICT,
    difficulty        text NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    publication_state text NOT NULL DEFAULT 'unpublished'
                          CHECK (publication_state IN ('unpublished', 'published')),
    published_version int,
    published_at      timestamptz,
    unpublished_at    timestamptz,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (publication_state <> 'published' OR published_version IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS problem_versions (
    problem_slug            text NOT NULL REFERENCES problems (slug) ON DELETE RESTRICT,
    version                 int  NOT NULL CHECK (version >= 1),
    content_sha256          bytea NOT NULL CHECK (octet_length(content_sha256) = 32),
    manifest                jsonb NOT NULL,
    title                   text NOT NULL CHECK (char_length(title) BETWEEN 1 AND 80),
    track_slug              text NOT NULL REFERENCES tracks (slug) ON DELETE RESTRICT,
    difficulty              text NOT NULL CHECK (difficulty IN ('easy', 'medium', 'hard')),
    statement_markdown      text NOT NULL CHECK (char_length(statement_markdown) <= 8000),
    constraints             jsonb NOT NULL CHECK (jsonb_typeof(constraints) = 'array'),
    signature               jsonb NOT NULL,
    input_schema            jsonb NOT NULL,
    comparator              text NOT NULL CHECK (comparator IN ('exact', 'unordered')),
    limits                  jsonb NOT NULL,
    reference_complexity    text,
    bench                   jsonb,
    complexity_note         text CHECK (char_length(complexity_note) <= 240),
    engine_version          text NOT NULL,
    generator_version       text NOT NULL,
    hidden_count            int NOT NULL CHECK (hidden_count BETWEEN 5 AND 80),
    statically_validated_at timestamptz NOT NULL DEFAULT now(),
    sandbox_validated_at    timestamptz,
    sandbox_image_digest    text CHECK (char_length(sandbox_image_digest) <= 200),
    created_at              timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (problem_slug, version),
    UNIQUE (problem_slug, content_sha256),
    CHECK ((sandbox_validated_at IS NULL) = (sandbox_image_digest IS NULL))
);

CREATE TABLE IF NOT EXISTS problem_concepts (
    problem_slug text NOT NULL REFERENCES problems (slug) ON DELETE RESTRICT,
    concept_id   text NOT NULL REFERENCES concepts (id) ON DELETE RESTRICT,
    PRIMARY KEY (problem_slug, concept_id)
);

CREATE TABLE IF NOT EXISTS problem_examples (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    ordinal      int  NOT NULL CHECK (ordinal >= 1),
    input        jsonb NOT NULL,
    expected     jsonb NOT NULL,
    PRIMARY KEY (problem_slug, version, ordinal),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS problem_starters (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    lang         text NOT NULL CHECK (lang IN ('python', 'javascript')),
    code         text NOT NULL CHECK (octet_length(code) <= 8192),
    PRIMARY KEY (problem_slug, version, lang),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS problem_hints (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    rung         int  NOT NULL CHECK (rung BETWEEN 1 AND 4),
    body         text NOT NULL CHECK (char_length(body) BETWEEN 1 AND 600),
    PRIMARY KEY (problem_slug, version, rung),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS hidden_tests (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    ordinal      int  NOT NULL CHECK (ordinal >= 1),
    source       text NOT NULL CHECK (source IN ('curated', 'generated')),
    input        jsonb NOT NULL,
    expected     jsonb NOT NULL,
    PRIMARY KEY (problem_slug, version, ordinal),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS hidden_generation (
    problem_slug      text NOT NULL,
    version           int  NOT NULL,
    generator_seed    bigint NOT NULL,
    generator_version text NOT NULL,
    PRIMARY KEY (problem_slug, version),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS hidden_sources (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    path         text NOT NULL CHECK (path LIKE 'private/%' AND char_length(path) <= 300),
    sha256       bytea NOT NULL CHECK (octet_length(sha256) = 32),
    PRIMARY KEY (problem_slug, version, path),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS reference_solutions (
    problem_slug text NOT NULL,
    version      int  NOT NULL,
    lang         text NOT NULL CHECK (lang = 'python'),
    code         text NOT NULL CHECK (octet_length(code) <= 262144),
    sha256       bytea NOT NULL CHECK (octet_length(sha256) = 32),
    PRIMARY KEY (problem_slug, version, lang),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS known_ghosts (
    problem_slug    text NOT NULL,
    version         int  NOT NULL,
    ghost_id        text NOT NULL CHECK (char_length(ghost_id) BETWEEN 1 AND 60),
    kind            text NOT NULL CHECK (kind IN ('wrong', 'crash', 'timeout')),
    input           jsonb NOT NULL,
    got             jsonb,
    exception_class text CHECK (char_length(exception_class) <= 120),
    explanation     text NOT NULL CHECK (octet_length(explanation) <= 400),
    concepts        text[] NOT NULL DEFAULT '{}',
    PRIMARY KEY (problem_slug, version, ghost_id),
    FOREIGN KEY (problem_slug, version) REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS catalogue_events (
    id           bigserial PRIMARY KEY,
    problem_slug text NOT NULL REFERENCES problems (slug) ON DELETE RESTRICT,
    action       text NOT NULL CHECK (action IN ('import', 'sandbox_validate', 'publish', 'unpublish')),
    version      int,
    operator     text CHECK (char_length(operator) <= 60),
    detail       jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at   timestamptz NOT NULL DEFAULT now()
);
