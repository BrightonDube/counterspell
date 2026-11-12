-- Minimal bootstrap migration. Domain tables belong to later feature specs.
CREATE TABLE IF NOT EXISTS counterspell_schema_metadata (
    key text PRIMARY KEY,
    value text NOT NULL
);

INSERT INTO counterspell_schema_metadata (key, value)
VALUES ('skeleton_version', '0.1.0')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
