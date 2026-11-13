-- Demo daily reset with application time (auth T28). Replaces the zero-argument function from
-- 20261113110200_auth_demo.sql. The caller passes its clock as p_now and the scheduled reset time
-- as p_due. The function resets only when the stored reset is older than p_due, so a restart or a
-- concurrent check cannot reset twice. Both parameters default to now(), so a plain
-- `SELECT cs_reset_demo_account()` behaves as before.
-- Like the earlier version, it never builds SQL from user input.

DROP FUNCTION IF EXISTS cs_reset_demo_account();

CREATE FUNCTION cs_reset_demo_account(p_now timestamptz DEFAULT now(), p_due timestamptz DEFAULT now())
RETURNS bigint
LANGUAGE plpgsql SECURITY DEFINER SET search_path = pg_catalog, public AS $$
DECLARE
    demo uuid;
    last_reset timestamptz;
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

    -- Read the stamp only after taking the lock, so a second caller sees the reset of the first.
    SELECT demo_reset_at INTO last_reset FROM users WHERE id = demo FOR UPDATE;
    IF last_reset IS NOT NULL AND last_reset >= p_due THEN
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
           -- Compare OIDs, not names: regclass::text drops the schema, so a name list never matches.
           AND c.conrelid NOT IN (
               'public.sessions'::regclass, 'public.identities'::regclass,
               'public.email_tokens'::regclass, 'public.oauth_flows'::regclass,
               'public.submissions'::regclass)
    LOOP
        EXECUTE format('DELETE FROM %s WHERE %I = $1', fk.tbl, fk.col) USING demo;
    END LOOP;

    UPDATE users
       SET xp = 0,
           streak_days = 0,
           last_active = NULL,
           familiar_prefs = '{}'::jsonb,
           demo_reset_at = p_now,
           updated_at = p_now
     WHERE id = demo;

    RETURN removed;
END
$$;

REVOKE ALL ON FUNCTION cs_reset_demo_account(timestamptz, timestamptz) FROM PUBLIC;
REVOKE ALL ON FUNCTION cs_reset_demo_account(timestamptz, timestamptz) FROM counterspell_judge;
GRANT EXECUTE ON FUNCTION cs_reset_demo_account(timestamptz, timestamptz) TO counterspell_server;
