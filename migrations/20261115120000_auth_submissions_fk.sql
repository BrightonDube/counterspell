-- Auth T08: a submission belongs to a real account. Deleting the account deletes its submissions.
-- Guarded so it does nothing when run-lifecycle has not created `submissions`, and so a second run
-- is a no-op. Existing rows must belong to real users, or this migration fails loudly and changes
-- nothing.

DO $$
BEGIN
    IF to_regclass('public.submissions') IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM pg_constraint WHERE conname = 'submissions_user_fk'
       )
    THEN
        ALTER TABLE submissions
            ADD CONSTRAINT submissions_user_fk
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;
    END IF;
END
$$;
