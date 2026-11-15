# Auth setup

This page covers the settings the server needs for sign-in. The server refuses to start when a
setting is missing or breaks a rule. Values come from the host environment. Never write them into
this repository, into `.env` files that are committed, or into logs.

## Settings

| Variable | Purpose |
|---|---|
| `APP_ENV` | `production` (default in Compose) or `development`. Development allows `http` on localhost and the log mail transport. |
| `PUBLIC_BASE_URL` | The site origin, for example `https://counterspell.example`. Every link and callback URL is built from it. |
| `SESSION_SECRET` | A random secret of at least 32 characters. It keys the rate-limit HMACs. |
| `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET` | GitHub sign-in. Both are needed to turn GitHub on. |
| `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET` | Google sign-in. Both are needed to turn Google on. |
| `MAIL_TRANSPORT` | `resend` in production. `log` is for development only and is refused in production. |
| `RESEND_API_KEY`, `MAIL_FROM` | Resend sending. Both are required when the transport is `resend`. |
| `EMAIL_DAILY_CAP` | The global daily mail cap. |
| `DEMO_ACCOUNT_EMAIL`, `DEMO_ACCOUNT_PASSWORD` | The demo account. Both are needed to turn it on. The password is 16 characters or more. |
| `DEMO_RESET_HOUR_UTC` | The hour (0 to 23) when the demo data resets each day. |

`docker-compose.yml` passes each of these to the `server` service only. The `judge` service receives
none of them.

## OAuth app registration

Register one OAuth app per provider. Use these callback URLs, where the origin is `PUBLIC_BASE_URL`:

- GitHub: `${PUBLIC_BASE_URL}/api/auth/oauth/github/callback`
- Google: `${PUBLIC_BASE_URL}/api/auth/oauth/google/callback`

Each provider uses one callback URL for both sign-in and linking. The stored flow decides which one
runs.

## Resend

Verify the sending domain in Resend and set `MAIL_FROM` to an address on that domain. SPF and DKIM
must both pass for the domain before real sign-in mail is tested.

## Demo account

Set `DEMO_ACCOUNT_EMAIL` and `DEMO_ACCOUNT_PASSWORD`. The server seeds the account at start-up and
resets its data each day at `DEMO_RESET_HOUR_UTC`. The demo account cannot link providers, change its
credentials or delete itself. The credentials go only in the private field of the external submission
form.

## Checks after deploy

- `GET /api/auth/config` lists only the enabled providers and the demo flag.
- A sign-in link arrives by mail and signs in once.
- A second use of the same link fails.
- A cross-site `POST` to `/api/auth/logout` is refused with `403`.
