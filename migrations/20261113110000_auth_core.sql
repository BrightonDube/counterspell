-- Auth core tables (auth T07). The Rust code in crates/server/src/auth relies on these names,
-- columns and CHECK values. Change them only together with .kiro/specs/auth/design.md.
-- Postgres is the only store for users, identities, sessions, email tokens and OAuth flows.
-- No secret, seed or password lives in this file.

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE IF NOT EXISTS users (
    id                 uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Only ever a verified address. The CHECKs below enforce it.
    email              citext UNIQUE,
    email_verified_at  timestamptz,
    password_hash      text,
    handle             citext NOT NULL UNIQUE CHECK (handle ~ '^[a-z0-9][a-z0-9_-]{2,31}$'),
    display_name       text NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 80),
    country            text CHECK (country IS NULL OR country ~ '^[A-Z]{2}$'),
    xp                 int NOT NULL DEFAULT 0 CHECK (xp >= 0),
    streak_days        int NOT NULL DEFAULT 0 CHECK (streak_days >= 0),
    last_active        date,
    familiar_prefs     jsonb NOT NULL DEFAULT '{}'::jsonb,
    is_demo            boolean NOT NULL DEFAULT false,
    -- Meaningful only on the demo row. Written by cs_reset_demo_account().
    demo_reset_at      timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT users_email_is_verified CHECK (email IS NULL OR email_verified_at IS NOT NULL),
    CONSTRAINT users_password_needs_email CHECK (password_hash IS NULL OR email IS NOT NULL),
    CONSTRAINT users_email_shape CHECK (
        email IS NULL OR (char_length(email) <= 254 AND email !~ '[[:space:][:cntrl:]<>,;"]')
    )
);

-- At most one judge demo account.
CREATE UNIQUE INDEX IF NOT EXISTS users_one_demo ON users ((true)) WHERE is_demo;

CREATE TABLE IF NOT EXISTS identities (
    user_id           uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider          text NOT NULL CHECK (provider IN ('github', 'google')),
    provider_user_id  text NOT NULL CHECK (char_length(provider_user_id) BETWEEN 1 AND 128),
    -- Display text only. Never used to find an account.
    provider_label    text CHECK (provider_label IS NULL OR char_length(provider_label) <= 100),
    linked_at         timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, provider_user_id),
    UNIQUE (user_id, provider)
);

CREATE TABLE IF NOT EXISTS email_tokens (
    token_hash  bytea PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    purpose     text NOT NULL CHECK (purpose IN ('magic_link', 'verify', 'reset', 'link_email')),
    email       citext NOT NULL CHECK (char_length(email) <= 254),
    user_id     uuid REFERENCES users(id) ON DELETE CASCADE,
    payload     jsonb NOT NULL DEFAULT '{}'::jsonb,
    -- No default: the application writes the Clock's time, so tests and checks agree.
    created_at  timestamptz NOT NULL,
    expires_at  timestamptz NOT NULL,
    used_at     timestamptz,
    CONSTRAINT email_tokens_ttl CHECK (
        expires_at > created_at AND expires_at <= created_at + interval '15 minutes'
    ),
    CONSTRAINT email_tokens_link_has_user CHECK (purpose <> 'link_email' OR user_id IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS email_tokens_live ON email_tokens (email, purpose) WHERE used_at IS NULL;
CREATE INDEX IF NOT EXISTS email_tokens_expiry ON email_tokens (expires_at);

CREATE TABLE IF NOT EXISTS sessions (
    -- SHA-256 hex of the cookie value. The cookie value itself is never stored.
    id                   text PRIMARY KEY CHECK (id ~ '^[0-9a-f]{64}$'),
    user_id              uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_method          text NOT NULL CHECK (auth_method IN (
                             'magic_link', 'oauth_github', 'oauth_google', 'password', 'register')),
    data                 jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at           timestamptz NOT NULL,
    last_renewed_at      timestamptz NOT NULL,
    expires_at           timestamptz NOT NULL,
    absolute_expires_at  timestamptz NOT NULL,
    CHECK (expires_at <= absolute_expires_at),
    CHECK (absolute_expires_at <= created_at + interval '90 days')
);
CREATE INDEX IF NOT EXISTS sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS sessions_expiry ON sessions (expires_at);

-- A session's owner, creation time and absolute limit never change, and its expiry only moves
-- forward. Renewal may move expires_at forward and nothing else.
CREATE OR REPLACE FUNCTION cs_guard_session_update() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.user_id IS DISTINCT FROM OLD.user_id
        OR NEW.created_at IS DISTINCT FROM OLD.created_at
        OR NEW.absolute_expires_at IS DISTINCT FROM OLD.absolute_expires_at
        OR NEW.expires_at < OLD.expires_at THEN
        RAISE EXCEPTION 'session ownership, creation time and limits cannot change'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END
$$;

DROP TRIGGER IF EXISTS sessions_guard_update ON sessions;
CREATE TRIGGER sessions_guard_update
    BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION cs_guard_session_update();

CREATE TABLE IF NOT EXISTS oauth_flows (
    state_hash     bytea PRIMARY KEY CHECK (octet_length(state_hash) = 32),
    provider       text NOT NULL CHECK (provider IN ('github', 'google')),
    purpose        text NOT NULL CHECK (purpose IN ('login', 'link')),
    pkce_verifier  text NOT NULL CHECK (char_length(pkce_verifier) BETWEEN 43 AND 128),
    binding_hash   bytea CHECK (binding_hash IS NULL OR octet_length(binding_hash) = 32),
    session_id     text REFERENCES sessions(id) ON DELETE CASCADE,
    user_id        uuid REFERENCES users(id) ON DELETE CASCADE,
    return_to      text NOT NULL DEFAULT '/' CHECK (char_length(return_to) <= 200),
    created_at     timestamptz NOT NULL,
    expires_at     timestamptz NOT NULL,
    CHECK (expires_at > created_at AND expires_at <= created_at + interval '10 minutes'),
    CHECK (
        (purpose = 'login' AND binding_hash IS NOT NULL AND session_id IS NULL AND user_id IS NULL)
        OR
        (purpose = 'link' AND binding_hash IS NULL AND session_id IS NOT NULL AND user_id IS NOT NULL)
    )
);
