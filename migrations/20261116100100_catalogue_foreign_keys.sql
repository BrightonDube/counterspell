-- Catalogue foreign keys (problem-catalogue C03, CAT-F1 to CAT-F3, CAT-S8, CAT-S9).
-- Idempotent: every constraint is added only when its name is missing.
-- Submission keys are NOT VALID, so Postgres checks new rows only and does not scan
-- existing submissions at deploy time.

ALTER TABLE submissions ADD COLUMN IF NOT EXISTS problem_version int CHECK (problem_version >= 1);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'submissions_problem_fk') THEN
        ALTER TABLE submissions
            ADD CONSTRAINT submissions_problem_fk
            FOREIGN KEY (problem_slug) REFERENCES problems (slug) ON DELETE RESTRICT NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'submissions_problem_version_fk') THEN
        -- A NULL problem_version (rows from before the catalogue) skips this check.
        ALTER TABLE submissions
            ADD CONSTRAINT submissions_problem_version_fk
            FOREIGN KEY (problem_slug, problem_version)
            REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT NOT VALID;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'problems_published_version_fk') THEN
        -- A NULL published_version (unpublished problems) skips this check.
        ALTER TABLE problems
            ADD CONSTRAINT problems_published_version_fk
            FOREIGN KEY (slug, published_version)
            REFERENCES problem_versions (problem_slug, version) ON DELETE RESTRICT;
    END IF;
END;
$$;

-- Guarded keys for tables that later specs create. Each is added only when its table exists.
DO $$
BEGIN
    IF to_regclass('public.problem_baselines') IS NOT NULL
       AND NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'problem_baselines_problem_fk') THEN
        ALTER TABLE problem_baselines
            ADD CONSTRAINT problem_baselines_problem_fk
            FOREIGN KEY (problem_slug) REFERENCES problems (slug) ON DELETE RESTRICT;
    END IF;
    IF to_regclass('public.concept_cards') IS NOT NULL
       AND NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'concept_cards_concept_fk') THEN
        ALTER TABLE concept_cards
            ADD CONSTRAINT concept_cards_concept_fk
            FOREIGN KEY (concept_id) REFERENCES concepts (id) ON DELETE RESTRICT;
    END IF;
END;
$$;
