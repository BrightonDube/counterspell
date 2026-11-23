# Monitoring

The exact URLs a probe may call, and the only replies that count as healthy.
SPA HTML (`text/html`) never counts as a healthy API response — a monitor that
accepts it is checking the wrong thing.

## Liveness — public

| Item | Value |
|---|---|
| URL | `https://counterspel.online/api/health` |
| Expected status | `200` |
| Expected content type | `application/json` |
| Expected body | `{"status":"ok"}` |
| Who calls it | External uptime probes; the deploy workflow's post-deploy check (`HEALTH_URL` in `.github/workflows/deploy.yml`) |

## Readiness — internal only

| Item | Value |
|---|---|
| URL | `/api/ready` on the origin (not the public hostname) |
| Public behaviour | Caddy answers `404` at the edge; the path never reaches the app from outside |
| Internal reply | `200 {"status":"ok"}` when Postgres and Valkey answer within 2 s each, otherwise `503 {"status":"unavailable"}` |
| Who calls it | The deploy script inside the edge network; never an external probe |

Failing dependency names go to the server log at `warn`, never to the client.

## Look-alike paths

`GET /health`, `GET /ready`, `GET /healthz` and `GET /readiness` are not routes.
They return an honest `404` (`text/html` app shell) — never a healthy response.
