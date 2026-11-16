-- Judge demo account guards and daily reset (auth T09). The application checks capabilities first;
-- these triggers are the second barrier. No password, email or seed value is stored here.
-- The seed helper and the reset function are the only code paths that touch the demo account.

-- Barrier for the users table: the demo account can be created, re-credentialed and deleted
-- only through the seed helper, and its id and demo flag never change.
CREATE OR REPLACE FUNCTION cs_guard_demo_user() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.is_demo THEN
            RAISE EXCEPTION 'the demo account cannot be deleted' USING ERRCODE = 'check_violation';
        END IF;
        RETURN OLD;
    END IF;

    IF TG_OP = 'INSERT' AND NEW.is_demo
       AND coalesce(current_setting('counterspell.demo_seed', true), '') <> 'on' THEN
        RAISE EXCEPTION 'the demo account can only be created during seeding'
            USING ERRCODE = 'check_violation';
    END IF;

    IF TG_OP = 'UPDATE' AND NEW.is_demo AND NOT OLD.is_demo THEN
        RAISE EXCEPTION 'an existing account cannot become the demo account'
            USING ERRCODE = 'check_violation';
    END IF;

    IF TG_OP = 'UPDATE' AND OLD.is_demo THEN
        IF NEW.is_demo IS DISTINCT FROM true OR NEW.id <> OLD.id THEN
            RAISE EXCEPTION 'the demo account cannot lose its demo flag or id'
                USING ERRCODE = 'check_violation';
        END IF;
        IF (NEW.email IS DISTINCT FROM OLD.email
            OR NEW.password_hash IS DISTINCT FROM OLD.password_hash
            OR NEW.handle <> OLD.handle)
           AND coalesce(current_setting('counterspell.demo_seed', true), '') <> 'on' THEN
            RAISE EXCEPTION 'the demo account credentials can only change during seeding'
                USING ERRCODE = 'check_violation';
        END IF;
    END IF;

    RETURN NEW;
END
$$;

DROP TRIGGER IF EXISTS users_guard_demo ON users;
CREATE TRIGGER users_guard_demo
    BEFORE INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION cs_guard_demo_user();

-- No identity can be linked to the demo account, whatever the application does.
CREATE OR REPLACE FUNCTION cs_guard_demo_identity() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (SELECT 1 FROM users WHERE id = NEW.user_id AND is_demo) THEN
        RAISE EXCEPTION 'the demo account cannot link an identity' USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END
$$;

DROP TRIGGER IF EXISTS identities_guard_demo ON identities;
CREATE TRIGGER identities_guard_demo
    BEFORE INSERT ON identities
    FOR EACH ROW EXECUTE FUNCTION cs_guard_demo_identity();

-- Creates or refreshes the single demo account. The setting is local to the transaction, so it
-- cannot leak into other statements. The fixed search_path stops a caller from shadowing names.
CREATE OR REPLACE FUNCTION cs_seed_demo_user(p_email text, p_password_hash text) RETURNS uuid
LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, public AS $$
DECLARE
    demo_id uuid;
BEGIN
    PERFORM set_config('counterspell.demo_seed', 'on', true);

    SELECT id INTO demo_id FROM users WHERE is_demo;
    IF demo_id IS NULL THEN
        INSERT INTO users (email, email_verified_at, password_hash, handle, display_name, is_demo)
        VALUES (p_email::citext, now(), p_password_hash, 'demo', 'Judge demo', true)
        RETURNING id INTO demo_id;
    ELSE
        UPDATE users
           SET email = p_email::citext,
               email_verified_at = coalesce(email_verified_at, now()),
               password_hash = p_password_hash,
               updated_at = now()
         WHERE id = demo_id;
    END IF;
    RETURN demo_id;
END
$$;

-- Clears the demo account's finished work and counters. Running work is kept, so the judge never
-- loses a row mid-run. Every other single-column foreign key to users is cleared for the demo
-- user only. No user text reaches the dynamic SQL: table and column names come from the catalogue
-- and are quoted with format().
CREATE OR REPLACE FUNCTION cs_reset_demo_account() RETURNS bigint
LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, public AS $$
DECLARE
    demo uuid;
    fk record;
    removed bigint := 0;
    affected bigint;
BEGIN
    SELECT id INTO demo FROM users WHERE is_demo;
    IF demo IS NULL THEN
        RETURN 0;
    END IF;
    IF NOT pg_try_advisory_xact_lock(hashtextextended('cs_demo_reset', 0)) THEN
        RETURN 0;
    END IF;

    IF to_regclass('public.submissions') IS NOT NULL THEN
        EXECUTE 'DELETE FROM public.submissions WHERE user_id = $1 AND finished_at IS NOT NULL'
            USING demo;
        GET DIAGNOSTICS affected = ROW_COUNT;
        removed := removed + affected;
    END IF;

    FOR fk IN
        SELECT c.conrelid::regclass AS tbl, a.attname AS col
          FROM pg_constraint c
          JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = c.conkey[1]
         WHERE c.contype = 'f'
           AND c.confrelid = 'public.users'::regclass
           AND array_length(c.conkey, 1) = 1
           AND c.conrelid::regclass::text NOT IN (
               'public.sessions', 'public.identities', 'public.email_tokens',
               'public.oauth_flows', 'public.submissions')
    LOOP
        EXECUTE format('DELETE FROM %s WHERE %I = $1', fk.tbl, fk.col) USING demo;
    END LOOP;

    UPDATE users
       SET xp = 0,
           streak_days = 0,
           last_active = NULL,
           familiar_prefs = '{}'::jsonb,
           demo_reset_at = now(),
           updated_at = now()
     WHERE id = demo;

    RETURN removed;
END
$$;

REVOKE ALL ON FUNCTION cs_seed_demo_user(text, text) FROM PUBLIC;
REVOKE ALL ON FUNCTION cs_reset_demo_account() FROM PUBLIC;
REVOKE ALL ON FUNCTION cs_seed_demo_user(text, text) FROM counterspell_judge;
REVOKE ALL ON FUNCTION cs_reset_demo_account() FROM counterspell_judge;
GRANT EXECUTE ON FUNCTION cs_seed_demo_user(text, text) TO counterspell_server;
GRANT EXECUTE ON FUNCTION cs_reset_demo_account() TO counterspell_server;
