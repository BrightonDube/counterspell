-- Language consistency LC06: a version must carry a starter for every accepted
-- launch language (python, javascript, java, cpp) before a problem may publish it.
-- The set mirrors problem_starters_lang_check; the database stays the authority
-- on the gate, so the check lives in the trigger, not the importer.
CREATE OR REPLACE FUNCTION catalogue_publish_gate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    v_validated timestamptz;
    v_examples  int;
    v_hints     int;
    v_concepts  int;
    v_starters  int;
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
    SELECT count(DISTINCT lang) INTO v_starters FROM problem_starters
    WHERE problem_slug = NEW.slug AND version = NEW.published_version
      AND lang IN ('python', 'javascript', 'java', 'cpp');
    IF v_starters < 4 THEN
        RAISE EXCEPTION 'publish refused: needs a starter for every launch language, found % of 4', v_starters;
    END IF;
    RETURN NEW;
END;
$$;
