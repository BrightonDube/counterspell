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

-- ---------------------------------------------------------------------------
-- Triggers (problem-catalogue C02, design "Immutability trigger" and "Publish gate").
-- ---------------------------------------------------------------------------

-- Append-only rows: any update or delete raises. Used by child tables and events.
CREATE OR REPLACE FUNCTION catalogue_append_only() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'append-only catalogue row: % cannot be % (CAT-S2)', TG_TABLE_NAME, lower(TG_OP);
END;
$$;

-- Child rows are insert-only, and only while their version has no sandbox stamp (CAT-S3).
CREATE OR REPLACE FUNCTION catalogue_child_insert_gate() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM problem_versions v
        WHERE v.problem_slug = NEW.problem_slug
          AND v.version = NEW.version
          AND v.sandbox_validated_at IS NOT NULL
    ) THEN
        RAISE EXCEPTION 'version is sandbox-validated and frozen: % refused (CAT-S3)', TG_TABLE_NAME;
    END IF;
    RETURN NEW;
END;
$$;

-- A version is immutable. The one exception is the one-time sandbox stamp (CAT-S2): the stamp
-- columns go from NULL to a value, every other column is unchanged, and hidden tests exist.
CREATE OR REPLACE FUNCTION catalogue_version_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'problem version is immutable and cannot be deleted (CAT-S2)';
    END IF;
    IF OLD.sandbox_validated_at IS NOT NULL OR OLD.sandbox_image_digest IS NOT NULL THEN
        RAISE EXCEPTION 'problem version is immutable once sandbox-validated (CAT-S2)';
    END IF;
    IF (to_jsonb(NEW) - 'sandbox_validated_at' - 'sandbox_image_digest')
       IS DISTINCT FROM (to_jsonb(OLD) - 'sandbox_validated_at' - 'sandbox_image_digest') THEN
        RAISE EXCEPTION 'problem version is immutable; only the sandbox stamp may change (CAT-S2)';
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM hidden_tests h
        WHERE h.problem_slug = NEW.problem_slug AND h.version = NEW.version
    ) THEN
        RAISE EXCEPTION 'sandbox fill needs hidden tests before the version is stamped (CAT-S2)';
    END IF;
    RETURN NEW;
END;
$$;

-- Publish gate (CAT-S4, D9): a version must be sandbox-validated, have 2 to 5 examples,
-- exactly 4 hints and at least one concept tag, before a problem can point at it.
CREATE OR REPLACE FUNCTION catalogue_publish_gate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    v_validated timestamptz;
    v_examples  int;
    v_hints     int;
    v_concepts  int;
BEGIN
    IF NEW.publication_state <> 'published' THEN
        RETURN NEW;
    END IF;
    SELECT sandbox_validated_at INTO v_validated
    FROM problem_versions
    WHERE problem_slug = NEW.slug AND version = NEW.published_version;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'publish refused: version % does not exist', NEW.published_version;
    END IF;
    IF v_validated IS NULL THEN
        RAISE EXCEPTION 'publish refused: version is not sandbox-validated';
    END IF;
    SELECT count(*) INTO v_examples FROM problem_examples
    WHERE problem_slug = NEW.slug AND version = NEW.published_version;
    IF v_examples NOT BETWEEN 2 AND 5 THEN
        RAISE EXCEPTION 'publish refused: needs 2 to 5 examples, found %', v_examples;
    END IF;
    SELECT count(*) INTO v_hints FROM problem_hints
    WHERE problem_slug = NEW.slug AND version = NEW.published_version;
    IF v_hints <> 4 THEN
        RAISE EXCEPTION 'publish refused: needs exactly four hints, found %', v_hints;
    END IF;
    SELECT count(*) INTO v_concepts FROM problem_concepts WHERE problem_slug = NEW.slug;
    IF v_concepts < 1 THEN
        RAISE EXCEPTION 'publish refused: needs at least one concept tag';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS problem_versions_guard ON problem_versions;
CREATE TRIGGER problem_versions_guard
    BEFORE UPDATE OR DELETE ON problem_versions
    FOR EACH ROW EXECUTE FUNCTION catalogue_version_guard();

DROP TRIGGER IF EXISTS problem_examples_append_only ON problem_examples;
CREATE TRIGGER problem_examples_append_only
    BEFORE UPDATE OR DELETE ON problem_examples
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS problem_examples_insert_gate ON problem_examples;
CREATE TRIGGER problem_examples_insert_gate
    BEFORE INSERT ON problem_examples
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS problem_starters_append_only ON problem_starters;
CREATE TRIGGER problem_starters_append_only
    BEFORE UPDATE OR DELETE ON problem_starters
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS problem_starters_insert_gate ON problem_starters;
CREATE TRIGGER problem_starters_insert_gate
    BEFORE INSERT ON problem_starters
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS problem_hints_append_only ON problem_hints;
CREATE TRIGGER problem_hints_append_only
    BEFORE UPDATE OR DELETE ON problem_hints
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS problem_hints_insert_gate ON problem_hints;
CREATE TRIGGER problem_hints_insert_gate
    BEFORE INSERT ON problem_hints
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS hidden_tests_append_only ON hidden_tests;
CREATE TRIGGER hidden_tests_append_only
    BEFORE UPDATE OR DELETE ON hidden_tests
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS hidden_tests_insert_gate ON hidden_tests;
CREATE TRIGGER hidden_tests_insert_gate
    BEFORE INSERT ON hidden_tests
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS hidden_generation_append_only ON hidden_generation;
CREATE TRIGGER hidden_generation_append_only
    BEFORE UPDATE OR DELETE ON hidden_generation
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS hidden_generation_insert_gate ON hidden_generation;
CREATE TRIGGER hidden_generation_insert_gate
    BEFORE INSERT ON hidden_generation
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS hidden_sources_append_only ON hidden_sources;
CREATE TRIGGER hidden_sources_append_only
    BEFORE UPDATE OR DELETE ON hidden_sources
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS hidden_sources_insert_gate ON hidden_sources;
CREATE TRIGGER hidden_sources_insert_gate
    BEFORE INSERT ON hidden_sources
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS reference_solutions_append_only ON reference_solutions;
CREATE TRIGGER reference_solutions_append_only
    BEFORE UPDATE OR DELETE ON reference_solutions
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS reference_solutions_insert_gate ON reference_solutions;
CREATE TRIGGER reference_solutions_insert_gate
    BEFORE INSERT ON reference_solutions
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS known_ghosts_append_only ON known_ghosts;
CREATE TRIGGER known_ghosts_append_only
    BEFORE UPDATE OR DELETE ON known_ghosts
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();
DROP TRIGGER IF EXISTS known_ghosts_insert_gate ON known_ghosts;
CREATE TRIGGER known_ghosts_insert_gate
    BEFORE INSERT ON known_ghosts
    FOR EACH ROW EXECUTE FUNCTION catalogue_child_insert_gate();

DROP TRIGGER IF EXISTS catalogue_events_append_only ON catalogue_events;
CREATE TRIGGER catalogue_events_append_only
    BEFORE UPDATE OR DELETE ON catalogue_events
    FOR EACH ROW EXECUTE FUNCTION catalogue_append_only();

DROP TRIGGER IF EXISTS problems_publish_gate ON problems;
CREATE TRIGGER problems_publish_gate
    BEFORE UPDATE OF publication_state, published_version ON problems
    FOR EACH ROW EXECUTE FUNCTION catalogue_publish_gate();
