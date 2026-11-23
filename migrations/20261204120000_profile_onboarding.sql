-- Profile onboarding completion (profile-onboarding PO-1).
-- Existing accounts are considered already onboarded; only newly created OAuth accounts start NULL.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS profile_completed_at timestamptz;

UPDATE users
    SET profile_completed_at = created_at
    WHERE profile_completed_at IS NULL;

GRANT UPDATE (profile_completed_at) ON users TO counterspell_server;
