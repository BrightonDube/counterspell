#!/bin/sh
# Runs once at first initdb inside the postgres container (docker-entrypoint-initdb.d/00-roles.sh).
# It creates the NOLOGIN group roles the migrations own, the per-service LOGIN roles and the
# memberships, so the server and judge can connect before the first migration exists. The
# migrations' own CREATE ROLE blocks tolerate the roles already being there (they catch
# duplicate_object) and then grant the table matrix. Passwords come from postgres.env at runtime;
# they never appear in this file.
set -eu

psql -v ON_ERROR_STOP=1 \
  -v server_pw="${CS_SERVER_PASSWORD:?CS_SERVER_PASSWORD is required}" \
  -v judge_pw="${CS_JUDGE_PASSWORD:?CS_JUDGE_PASSWORD is required}" \
  --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<'SQL'
CREATE ROLE counterspell_server NOLOGIN;
CREATE ROLE counterspell_judge NOLOGIN;
CREATE ROLE cs_server LOGIN PASSWORD :'server_pw';
CREATE ROLE cs_judge LOGIN PASSWORD :'judge_pw';
GRANT counterspell_server TO cs_server;
GRANT counterspell_judge TO cs_judge;
SQL
