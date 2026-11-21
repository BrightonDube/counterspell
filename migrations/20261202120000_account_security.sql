-- Only the auth server can manage sign-in factors and challenges.
CREATE TABLE auth_totp (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret bytea NOT NULL,
    last_step bigint NOT NULL,
    created_at timestamptz NOT NULL
);

CREATE TABLE auth_totp_setup (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    secret bytea NOT NULL,
    expires_at timestamptz NOT NULL
);

CREATE TABLE auth_recovery_codes (
    user_id uuid NOT NULL REFERENCES auth_totp(user_id) ON DELETE CASCADE,
    code_hash bytea NOT NULL CHECK (octet_length(code_hash) = 32),
    PRIMARY KEY (user_id, code_hash)
);

CREATE TABLE auth_step_up (
    token_hash bytea PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_method text NOT NULL,
    return_to text NOT NULL CHECK (char_length(return_to) <= 200),
    expires_at timestamptz NOT NULL
);
CREATE INDEX auth_step_up_user ON auth_step_up (user_id);
CREATE INDEX auth_step_up_expiry ON auth_step_up (expires_at);

CREATE TABLE auth_passkeys (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id bytea NOT NULL UNIQUE,
    credential jsonb NOT NULL,
    name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 64),
    created_at timestamptz NOT NULL
);
CREATE INDEX auth_passkeys_user ON auth_passkeys (user_id);

CREATE TABLE auth_passkey_flows (
    id uuid PRIMARY KEY,
    user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    session_id text REFERENCES sessions(id) ON DELETE CASCADE,
    kind text NOT NULL CHECK (kind IN ('register', 'login')),
    state jsonb NOT NULL,
    binding_hash bytea,
    name text,
    return_to text NOT NULL CHECK (char_length(return_to) <= 200),
    expires_at timestamptz NOT NULL
);
CREATE INDEX auth_passkey_flows_expiry ON auth_passkey_flows (expires_at);
CREATE INDEX auth_passkey_flows_session ON auth_passkey_flows (session_id);
CREATE INDEX auth_totp_setup_expiry ON auth_totp_setup (expires_at);

ALTER TABLE sessions
    ADD COLUMN second_factor_verified boolean NOT NULL DEFAULT false,
    ADD CONSTRAINT sessions_auth_method_expanded CHECK (auth_method IN (
        'magic_link', 'oauth_github', 'oauth_google', 'password', 'register', 'passkey'));
ALTER TABLE sessions DROP CONSTRAINT sessions_auth_method_check;

-- Keep MFA fail closed if a rollback runs a server that predates second-factor verification.
CREATE FUNCTION cs_require_session_factor() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM 1 FROM users WHERE id = NEW.user_id FOR NO KEY UPDATE;
    IF NOT NEW.second_factor_verified AND EXISTS (
        SELECT 1 FROM auth_totp WHERE user_id = NEW.user_id
    ) THEN
        RAISE EXCEPTION 'second factor required for a new session' USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER sessions_require_factor BEFORE INSERT ON sessions
    FOR EACH ROW EXECUTE FUNCTION cs_require_session_factor();

GRANT SELECT, INSERT, UPDATE, DELETE ON auth_totp, auth_totp_setup,
    auth_recovery_codes, auth_step_up, auth_passkeys, auth_passkey_flows TO counterspell_server;
GRANT UPDATE (second_factor_verified) ON sessions TO counterspell_server;
