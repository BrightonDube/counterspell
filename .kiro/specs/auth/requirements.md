# Requirements: auth

## Introduction

This spec defines how a person proves who they are to Counterspell, how that proof becomes a session, and how one person's account can gain more sign-in methods without ever being merged with someone else's.

Three sign-in methods stay, as decided: **GitHub and Google OAuth** (authorization code with `state` and PKCE), and **email** (a single-use magic link, plus email and password). A **judge demo account** gives hackathon judges a no-friction path. Sessions are opaque cookies backed by Postgres. Email is sent through **Resend**.

The rule that shapes everything: **an account is never linked, merged or signed in to because two things share an email address.** Linking happens only when a signed-in user starts it, and only after the new method is proven (OAuth with a link-intent `state` bound to their session, or a verification email).

Property-based testing (PBT) is **enabled** for this spec. The EARS criteria below are written so that each important one becomes a property in `design.md` (token single-use, expiry, session isolation, "an OAuth email match never links or signs in to another account", and more).

Sources: `docs/system-design.md` §1.1 (F1), §1.2, §4 (`users`, `identities`, `email_tokens`, `sessions`), §7.1, §9 (auth endpoints, judge demo account), §10 (configuration); `docs/requirements.md` ACC-1 to ACC-6 and their EARS criteria; steering `security.md`, `api-conventions.md`, `structure.md`, `tech.md`, `rust-teaching.md`; sibling specs `repo-skeleton` and `run-lifecycle` (the `UserResolver` seam, the database roles, rate-limit conventions).

## Glossary

- **Account / user**: one row in `users`. A person has one account.
- **Identity**: one row in `identities`: a provider (`github` or `google`) plus the provider's own stable id for the person. It is how an OAuth sign-in finds its account.
- **Email credential**: the address in `users.email`. It is the account's mailbox sign-in method (magic link, password). Only a **verified** address is ever stored there.
- **Mailbox proof**: the person opened a link that was emailed to the address.
- **Email token**: a row in `email_tokens`. Purposes: `magic_link`, `verify` (finish a registration), `reset` (password reset), `link_email` (add an email to a signed-in account).
- **Flow**: a row in `oauth_flows`: one OAuth round trip in progress. Purpose `login` or `link`.
- **State / PKCE verifier**: the random values that tie an OAuth callback to the flow that started it.
- **Binding cookie**: a short-lived cookie that ties a `login` flow to the browser that started it.
- **Link intent**: a `link` flow, created by a signed-in user and bound to that user's session.
- **Session**: a row in `sessions`; the browser holds an opaque random value, the database holds only its hash.
- **Session key**: the SHA-256 hex of the session value; the `sessions.id`.
- **Capabilities**: the actions an account may perform (link a provider, change credentials, delete itself, sign out everywhere). Computed from the account; sent to the UI.
- **Demo account**: the single seeded account for judges (`users.is_demo`).
- **Clock**: the one injectable source of "now" used for every expiry decision. Tests use a manual clock.
- **Fake provider / fake mailer**: test doubles for an OAuth provider and for Resend. No test calls the internet.
- **Seam**: a trait owned by one spec and implemented by another. This spec implements `UserResolver` from `run-lifecycle`.

## Scope constraints

- The Cargo workspace keeps exactly the six canonical packages. This spec adds **no** package and no crate. New Rust code goes into `crates/server/src/auth/`. `counterspell-common` gets no auth-specific dependency.
- New database objects go into new files in `migrations/`. New web code goes into `web/src/routes/` and `web/src/lib/auth/`.
- **Postgres is the only store** for users, identities, sessions, tokens and flows. **Valkey holds rate-limit counters only.** No session, token or flow state is ever kept in Valkey or in process memory as the source of truth.
- Only `counterspell-server` touches auth data. `counterspell-judge`, `counterspell-mcp` and the sandbox get no auth credentials and no privilege on auth tables.
- Secrets are read from **environment variables only**. No task of this spec reads, writes or creates `.env`, key files or credential files.
- Not in this spec: roles and admin, account suspension (ADM-3), profile editing, **unlinking** a provider, **changing** an email address, multi-factor, passkeys, organisation SSO, payments, the run endpoints themselves, the IDE.
- Numbers marked **[E]** are estimates and configuration defaults. Tune them after the VPS test.
- Priority labels follow `docs/requirements.md` §8: **Must**, **Should**, **Could**.

## Requirements

### R1. Boundaries and storage (Must)

**User story:** As the operator, I want authentication data in one place that only the web server can reach, so that a compromised worker or a cache outage cannot expose or forge identities.

1. THE SYSTEM SHALL store users, identities, sessions, email tokens and OAuth flows only in Postgres.
2. THE SYSTEM SHALL use Valkey only for rate-limit counters. Keys SHALL hold a counter and nothing else (no email, no token, no session value).
3. THE database role used by `counterspell-judge` SHALL have no privilege of any kind on `users`, `identities`, `sessions`, `email_tokens` and `oauth_flows`.
4. THE SYSTEM SHALL keep all authentication code in `counterspell-server`. The judge, MCP server and sandbox SHALL receive no OAuth secret, Resend key, session secret or demo password.
5. THE SYSTEM SHALL NOT add a Cargo package or move responsibilities between the canonical packages.

### R2. Configuration and secrets (Must)

**User story:** As the operator, I want secrets to come only from the environment and to fail loudly when missing, so that nothing is committed and nothing half-works.

1. THE server SHALL read these variables: `PUBLIC_BASE_URL`, `DATABASE_URL`, `VALKEY_URL`, `SESSION_SECRET`, `APP_ENV`, `GITHUB_OAUTH_CLIENT_ID`, `GITHUB_OAUTH_CLIENT_SECRET`, `GOOGLE_OAUTH_CLIENT_ID`, `GOOGLE_OAUTH_CLIENT_SECRET`, `RESEND_API_KEY`, `MAIL_FROM`, `MAIL_TRANSPORT`, `EMAIL_DAILY_CAP`, `DEMO_ACCOUNT_EMAIL`, `DEMO_ACCOUNT_PASSWORD`, `DEMO_RESET_HOUR_UTC`.
2. IF `PUBLIC_BASE_URL`, `DATABASE_URL` or `SESSION_SECRET` is missing or malformed, THEN THE server SHALL refuse to start with a message that names the variable and never prints its value.
3. A provider (GitHub or Google) SHALL be enabled only when both its client id and client secret are set. WHEN a provider is disabled, THEN its endpoints SHALL answer `404 provider_disabled` and `GET /api/auth/config` SHALL report it as disabled.
4. THE configuration types SHALL print `[redacted]` for every secret in `Debug` and `Display` output.
5. WHEN `APP_ENV` is `production` (the default when unset), THE server SHALL require an `https` `PUBLIC_BASE_URL`, a `SESSION_SECRET` of at least 32 bytes, and `MAIL_TRANSPORT=resend` with `RESEND_API_KEY` and `MAIL_FROM` set whenever email sign-in is enabled. The `log` mail transport SHALL be refused in production.
6. WHERE `APP_ENV` is `development` and the base URL host is `localhost` or `127.0.0.1`, THE server MAY use an `http` base URL, non-Secure cookies without the `__Host-` prefix, and the `log` mail transport.
7. THE server SHALL build every absolute URL it emits (OAuth redirect URI, email links, redirects) from `PUBLIC_BASE_URL` only. It SHALL NOT use the `Host`, `X-Forwarded-Host` or `Forwarded` headers for this.
8. THE `SESSION_SECRET` SHALL be used only as the key for hashing rate-limit identifiers and log identifiers. It SHALL NOT sign or encrypt session cookies.

### R3. Single-use, expiring tokens (Must; ACC-1.2, ACC-1.3)

**User story:** As a user, I want a sign-in link to work once and soon stop working, so that an old email or a leaked link cannot be used against me.

1. THE SYSTEM SHALL generate every token (email token, OAuth `state`, binding value, session value) from the operating-system CSPRNG with at least 256 bits of entropy, encoded as unpadded base64url.
2. THE SYSTEM SHALL store only the SHA-256 hash of a token. The raw value SHALL exist only in the email link, the URL, or the cookie, and SHALL NOT be logged.
3. An email token SHALL be valid for exactly 15 minutes from issue. The database SHALL reject an email token whose lifetime exceeds 15 minutes. An OAuth flow SHALL be valid for at most 10 minutes.
4. WHEN a token is consumed, THE SYSTEM SHALL mark it used in the same atomic statement that checks it. IF the same token is presented again, or presented concurrently by several requests, THEN exactly one request SHALL succeed and every other SHALL fail.
5. IF a token is expired (`now >= expires_at`), used, unknown or issued for a different purpose, THEN THE SYSTEM SHALL answer with the same error `400 invalid_or_expired_token` and SHALL NOT reveal which case applied.
6. A token issued for one purpose SHALL NOT be accepted for another purpose.
7. WHEN a new token is issued for the same email and purpose, THE SYSTEM SHALL mark every earlier unused token for that email and purpose as unusable.
8. THE SYSTEM SHALL take "now" from one injectable Clock for issuing and checking, so that expiry decisions are deterministic in tests.
9. THE SYSTEM SHALL delete expired and used tokens and flows older than one day in a periodic cleanup.

### R4. Email delivery through Resend (Must for the magic link)

**User story:** As a user, I want the emailed link to arrive and be safe to click, so that I can sign in without a password.

1. THE SYSTEM SHALL send mail through an `EmailSender` interface. The production implementation SHALL call the Resend HTTP API with `RESEND_API_KEY` as a bearer token, `MAIL_FROM` as the sender, exactly one recipient per message, and an `Idempotency-Key` header derived from the token hash.
2. THE implementation SHALL use a connect timeout of 3 s and a total timeout of 10 s **[E]**, and SHALL retry at most once, only on a connection failure or a `5xx` answer.
3. THE SYSTEM SHALL validate and normalise an address before use: trim whitespace, lower-case it, ASCII only, at most 254 characters, exactly one `@`, and no whitespace, control character, `<`, `>`, `,`, `;` or `"`. IF the address is invalid, THEN THE SYSTEM SHALL answer `400 invalid_email`.
4. THE SYSTEM SHALL NOT canonicalise addresses beyond case (no dot-removal, no `+tag` stripping), so that two different mailboxes are never treated as one.
5. THE email link SHALL point to `PUBLIC_BASE_URL` and carry the token in the URL **fragment** (`/auth/verify#token=…`), so that it is not sent to servers, logs or `Referer`.
6. THE email body SHALL be built from fixed templates. Any value placed in the HTML part SHALL be HTML-escaped.
7. THE SYSTEM SHALL NOT send mail to the demo account's address.
8. WHEN the global daily email cap (default 80, below the Resend free plan's 100 per day **[E]**) is reached, THEN THE SYSTEM SHALL answer email-sending endpoints with `503 email_unavailable` and send nothing. Other sign-in methods SHALL keep working.
9. IF Resend rejects or fails a message after the request was accepted, THEN THE SYSTEM SHALL log the failure without the token and SHALL NOT change the response already sent.

### R5. Magic-link sign-in (Must; ACC-1.2, ACC-1.3)

**User story:** As a visitor, I want to enter my email and click the link I receive, so that I can start practising in under a minute.

1. WHEN a visitor submits `POST /api/auth/email/magic {email, return_to?}` with a valid address, THE SYSTEM SHALL answer `202 {"status":"sent"}` with identical status, body and headers whether or not an account exists for that address.
2. THE SYSTEM SHALL issue a `magic_link` token for the normalised address and send the link.
3. THE link SHALL open a web page that shows a confirmation button. A `GET` request SHALL NOT consume the token. The button SHALL call `POST /api/auth/email/verify {token}`.
4. WHEN a valid `magic_link` token is consumed and an account has that email, THEN THE SYSTEM SHALL sign in to that account.
5. WHEN a valid `magic_link` token is consumed and no account has that email, THEN THE SYSTEM SHALL create an account (verified email, generated handle) and sign in, in one transaction. Concurrent verification SHALL NOT create two accounts.
6. THE only address used SHALL be the one stored with the token. No request field SHALL choose the account.
7. IF the token is invalid, THEN THE SYSTEM SHALL create no session and the page SHALL offer to send a new link.
8. THE SYSTEM SHALL NOT sign in to the demo account by magic link.
9. THE `return_to` value, if present, SHALL pass the same-origin check in R6.8 and be stored with the token, not in the URL.

### R6. OAuth sign-in with GitHub and Google (Must; ACC-1.1)

**User story:** As a visitor, I want to sign in with GitHub or Google, so that I do not need another password.

1. WHEN a visitor opens `GET /api/auth/oauth/{github|google}/start`, THE SYSTEM SHALL create a `login` flow with a fresh 256-bit `state` and a PKCE verifier, set a binding cookie (HttpOnly, Secure, SameSite=Lax, 10 minutes), and redirect (`303`) to the provider with `state`, `code_challenge`, `code_challenge_method=S256` and a `redirect_uri` built from `PUBLIC_BASE_URL`.
2. THE requested scopes SHALL be the minimum: GitHub `read:user user:email`; Google `openid email profile`.
3. WHEN the provider redirects to the callback, THE SYSTEM SHALL check, in this order and before spending the authorization code: the provider is enabled; the `state` matches an unexpired flow for that provider (the flow is consumed atomically and cannot be reused, even if a later step fails); the binding cookie matches the flow (constant-time comparison).
4. THE SYSTEM SHALL then exchange the code together with the stored PKCE verifier and fetch the profile. A failure at any step SHALL redirect to `/login?error=<code>` and SHALL create no session.
5. IF the provider returns an `error` parameter such as `access_denied`, THEN THE SYSTEM SHALL consume the flow if it can be found and redirect to `/login?error=oauth_denied`.
6. THE account key SHALL be the pair (provider, provider's stable id): the numeric `id` for GitHub and `sub` for Google. Login names, display names and emails SHALL NOT be used as keys.
7. THE SYSTEM SHALL use a provider email only when the provider marks it verified (GitHub: primary and verified; Google: `email_verified` is true). An unverified email SHALL be treated as absent.
8. A `return_to` value SHALL be accepted only if it is a same-origin relative path: it starts with exactly one `/`, is not `//…` or `/\…`, still is not after percent-decoding, has no control character, and is at most 200 characters. Otherwise THE SYSTEM SHALL use `/`. The accepted value SHALL be stored in the flow, not carried in the URL.
9. Outbound provider calls SHALL verify TLS, use timeouts (3 s connect, 10 s total **[E]**), read at most 64 KiB of response, and not follow redirects on the token endpoint.
10. THE provider's access token SHALL be used once and SHALL NOT be stored or logged.
11. ON success, THE SYSTEM SHALL create a new session (R10), clear the binding cookie and redirect `303` to the stored `return_to`.

### R7. No automatic linking by email (Must; ACC-1.4)

**User story:** As a user, I want nobody to gain access to my account just because they control a matching email at some provider, so that my account cannot be taken over by an email match.

1. WHEN an OAuth callback succeeds and an identity with the same (provider, provider id) exists, THEN THE SYSTEM SHALL sign in to the account that owns that identity, whatever email the provider reports.
2. WHEN no such identity exists and the provider's verified email equals the `email` of any existing account, THEN THE SYSTEM SHALL create no session, create no identity, change no existing row, and redirect to `/login?error=account_exists`. The page SHALL tell the user to sign in with their original method and link the provider from account settings.
3. WHEN no such identity exists and the email is free or absent, THEN THE SYSTEM SHALL create a new account and its identity in one transaction. The account's `email` SHALL be set only if the provider's email is verified and free; otherwise it SHALL stay empty.
4. THE SYSTEM SHALL NOT look up, join or match accounts or identities by a provider-supplied email, except for the single "is this email taken" check in criterion 2.
5. FOR ALL sets of existing accounts and all provider profiles, an OAuth callback SHALL NOT create a session for an account whose only connection to the profile is a matching email.
6. THE address in `users.email` is the account's own email credential. A mailbox proof (R5) for that address signs in to that account. This is the account's own sign-in method and is not linking.
7. THE SYSTEM SHALL contain no code path that merges two accounts.

### R8. Linking a provider identity (Should; ACC-3.1, ACC-3.3)

**User story:** As a signed-in user, I want to add GitHub or Google to my account myself, so that I can sign in either way without anyone else being able to attach themselves to me.

1. WHEN a signed-in user sends `POST /api/auth/link/{github|google}/start`, THE SYSTEM SHALL create a `link` flow bound to that user's session and user id, and answer `200 {"authorize_url": …}` built as in R6.1.
2. WHEN the callback consumes a `link` flow, THE SYSTEM SHALL require the request's current session to be the very session bound to the flow. Otherwise it SHALL redirect to `/settings/account?error=state_invalid` and change nothing.
3. THE SYSTEM SHALL add the identity only after the provider exchange succeeds for that flow.
4. IF the identity is already linked to a different account, THEN THE SYSTEM SHALL refuse with `identity_already_linked` and change nothing. IF it is already linked to this account, THEN the result SHALL be a success without change.
5. IF the account already has an identity for that provider, THEN THE SYSTEM SHALL refuse with `provider_already_linked`.
6. THE provider's email SHALL NOT be used for linking and SHALL NOT need to match the account's email.
7. WHEN a session is deleted, THEN any `link` flow bound to it SHALL stop working.
8. THE SYSTEM SHALL refuse linking for the demo account (R16).

### R9. Linking an email address (Should; ACC-3.2)

**User story:** As a signed-in user whose account has no email, I want to add one after proving I own the mailbox, so that I can also sign in with a magic link.

1. WHEN a signed-in user sends `POST /api/auth/link/email {email}` and the account has no email, THE SYSTEM SHALL issue a `link_email` token bound to that user and send the link. The answer SHALL be `202` and identical whether or not the address belongs to another account (in that case no mail is sent).
2. IF the account already has an email, THEN THE SYSTEM SHALL answer `409 email_already_set`. Changing an email is out of scope.
3. WHEN `POST /api/auth/email/verify {token}` presents a `link_email` token, THE SYSTEM SHALL consume it only if the request carries a session of the **same user** the token was issued for. IF there is no session, THEN `401 unauthenticated`. IF the session belongs to another user, THEN `invalid_or_expired_token`. In both cases the token SHALL NOT be consumed.
4. WHEN consumed, THE SYSTEM SHALL set the account's email and `email_verified_at` in one transaction. IF the address was taken in the meantime, THEN `409 email_in_use` and no change.
5. THE link flow SHALL NOT create a session, change a session, or sign in anyone.
6. THE SYSTEM SHALL refuse this for the demo account (R16).

### R10. Sessions (Must; ACC-2)

**User story:** As a returning user, I want to stay signed in on my device, and I want a stolen database copy or a stale cookie to be useless to an attacker.

1. THE session cookie SHALL be `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`, and have no `Domain` attribute. WHERE the base URL is `https`, its name SHALL start with `__Host-`.
2. THE cookie value SHALL be a 256-bit random value. THE database SHALL store only its SHA-256 hex as the session key.
3. A session SHALL expire 30 days after its last renewal and, regardless of activity, 90 days after creation **[E]**. The database row is authoritative, not the cookie's `Max-Age`.
4. WHEN an authenticated request arrives and the last renewal is more than one hour old, THEN THE SYSTEM SHALL extend `expires_at` (not past the absolute limit) and re-issue the cookie.
5. IF a request carries a missing, unknown or expired session value, THEN THE SYSTEM SHALL treat it as signed out, answer `401 unauthenticated` on protected routes, and clear an unusable cookie.
6. WHEN a user signs in by any method, THE SYSTEM SHALL create a new session value. It SHALL NOT adopt a session value supplied by the client.
7. THE database SHALL refuse any change of a session's `user_id`.
8. WHEN an account is deleted, its sessions SHALL be deleted with it.
9. THE SYSTEM SHALL keep no application data in the session beyond who it belongs to and how it was created.

### R11. Logout and logout everywhere (Must for logout; Should for logout-all; ACC-4)

**User story:** As a user, I want to sign out of this device or all devices, so that a lost laptop is not a risk.

1. WHEN a client sends `POST /api/auth/logout`, THE SYSTEM SHALL delete the current session row, clear the cookie and answer `204`. Without a session it SHALL also answer `204`.
2. WHEN a signed-in user sends `POST /api/auth/logout-all`, THE SYSTEM SHALL delete every session of that user, including the current one, clear the cookie and answer `204`.
3. AFTER either call, THE SYSTEM SHALL reject the deleted sessions on their very next request with `401`.
4. A logout SHALL NOT affect any other user's sessions.
5. WHEN a password reset completes (R15.8), THE SYSTEM SHALL delete all of that user's sessions.
6. THE SYSTEM SHALL provide a way for a long-lived connection (such as an SSE stream) to learn that its session has been revoked, so that streams can stop within one heartbeat (15 s).
7. THE demo account SHALL be allowed `logout` and SHALL be refused `logout-all` (R16.4).

### R12. Identity of the caller and data isolation (Must)

**User story:** As a user, I want my data to be visible only to me, so that no request, however crafted, can read another person's account.

1. THE SYSTEM SHALL obtain the caller's `UserId` only from a valid session. No request body field, path segment, query parameter or header other than the session cookie SHALL select the user.
2. THE SYSTEM SHALL implement the `UserResolver` seam defined by `run-lifecycle`, so that the run, SSE and later endpoints use the session user.
3. WHEN a protected route is called without a valid session, THEN `401 unauthenticated` with a generic message.
4. `GET /api/me` SHALL return only the caller's data: `id`, `handle`, `display_name`, `email`, `email_verified`, `identities` (provider, linked_at, label), `is_demo`, `capabilities`. It SHALL NEVER return a password hash, token hash, session key or another user's data.
5. A session SHALL NEVER authorise reading or changing another user's rows. A resource owned by someone else SHALL be reported exactly like a missing one (`404`), as `run-lifecycle` already requires.
6. User ids exposed to clients SHALL be random UUIDs.

### R13. Request hardening and CSRF (Must)

**User story:** As a user, I want a malicious website unable to act as me, so that visiting it does no harm.

1. FOR every non-safe method (`POST`, `PUT`, `PATCH`, `DELETE`) under `/api`, THE SYSTEM SHALL require that: if an `Origin` header is present it equals the origin of `PUBLIC_BASE_URL`; otherwise if `Sec-Fetch-Site` is present it is `same-origin` or `none`; otherwise the request carries no `Cookie` header. A request that fails SHALL get `403 csrf_origin_mismatch` and change nothing.
2. THE SYSTEM SHALL require `Content-Type: application/json` on non-safe requests that have a body, and cap auth request bodies at 16 KiB.
3. Auth responses and `/api/me` SHALL carry `Cache-Control: no-store`.
4. Redirects emitted by auth routes SHALL carry `Referrer-Policy: no-referrer`.
5. THE middleware SHALL apply to every route under `/api`, including the run endpoints of `run-lifecycle`.
6. A `GET` request SHALL NOT change account data. The OAuth `start` and `callback` routes are the only `GET` routes allowed to create a flow or a session, because OAuth requires it.

### R14. Rate limits and abuse control (Must)

**User story:** As the operator, I want sign-in endpoints that cannot be used to guess passwords, burn the free email quota or exhaust the server.

1. THE SYSTEM SHALL enforce these fixed-window limits in Valkey **[E]**: magic-link, register and reset requests: 3 per hour per email and 10 per hour per IP; verify attempts: 30 per 10 minutes per IP; password login: 5 failures per 15 minutes per (email, IP) and 20 failures per 15 minutes per IP; OAuth start: 20 per 10 minutes per IP; link start: 10 per hour per user.
2. IF a limit is exceeded, THEN THE SYSTEM SHALL answer `429 rate_limited` with a `Retry-After` header and a `retry_after_seconds` field of at least 1, and SHALL NOT send mail or hash a password.
3. Valkey keys SHALL be namespaced (`rl:auth:<action>:<scope>:<id>`), expire with their window, and use a keyed hash (HMAC with `SESSION_SECRET`) of the email or IP, never the raw value.
4. IF Valkey is unavailable, THEN email-sending endpoints SHALL fail **closed** (`503 email_unavailable`, nothing sent) and the other limits SHALL fail **open** with a logged warning.
5. THE SYSTEM SHALL limit concurrent password hashing to a fixed number (default 4 **[E]**). IF no slot frees within 2 s, THEN `503 busy`.
6. THE client IP SHALL be read from the proxy header only when the connection comes from a configured trusted proxy (the rule of `run-lifecycle` R5.7).
7. THE global daily email cap of R4.8 SHALL be counted in Valkey and be shared by all email purposes.

### R15. Password sign-in (login: Must; register and reset: Should; F1, ACC-6)

**User story:** As a user who prefers a password, and as the demo account, I want to sign in with an email and a password, so that sign-in does not depend on my inbox.

1. WHEN a client sends `POST /api/auth/login {email, password}` with correct credentials, THE SYSTEM SHALL create a session and answer `200 {"user": …}`.
2. THE answer to an unknown email, a wrong password, and an account without a password SHALL be identical (`401 invalid_credentials`), and an unknown email SHALL still cost one dummy password verification.
3. THE SYSTEM SHALL hash passwords with Argon2id (at least 19 MiB memory, 2 iterations, 1 lane) stored as a PHC string, and SHALL re-hash after a successful login when the stored parameters are weaker.
4. A password SHALL be 10 to 128 characters and at most 1024 bytes. There SHALL be no composition rules. IF it is outside the limits, THEN `400 weak_password`.
5. THE SYSTEM SHALL run hashing and verification off the async runtime's worker threads.
6. A password and a request body containing one SHALL NOT be logged.
7. WHEN a client sends `POST /api/auth/register {email, password}`, THE SYSTEM SHALL answer `202` identically for any valid address, store the Argon2id hash in a `verify` token payload, send a link, and create **no account yet**. WHEN the token is consumed and no account has the email, THEN THE SYSTEM SHALL create the account with that password and sign in. IF an account already has the email, THEN it SHALL change nothing, sign in nobody and report `account_exists`.
8. WHEN a client sends `POST /api/auth/password/reset {email}`, THE SYSTEM SHALL answer `202` identically for any valid address and send mail only if the account exists, has a password, and is not the demo account. WHEN the `reset` token is consumed with `POST /api/auth/password/reset/confirm {token, new_password}`, THEN THE SYSTEM SHALL replace the hash, delete all of the user's sessions and sign in nobody.
9. A password SHALL be set only by criterion 7 (new account) or criterion 8 (existing password). No other path SHALL add a password to an account.

### R16. Judge demo account (Must; ACC-6)

**User story:** As a hackathon judge, I want to sign in with a given demo account in seconds and try everything, without being able to harm the account or other judges.

1. THE demo account SHALL exist only when `DEMO_ACCOUNT_EMAIL` and `DEMO_ACCOUNT_PASSWORD` (at least 16 characters) are set. Otherwise it SHALL be disabled and its login-page link hidden.
2. WHEN the server starts, THE SYSTEM SHALL seed the demo account idempotently: exactly one user with `is_demo = true`, handle `demo`, a verified email, and an Argon2id hash of the configured password. THE password SHALL NOT appear in the repository, in logs, in any response or in plain text in the database.
3. THE login page SHALL show a "Judging? Use the demo account" link that opens the demo sign-in form. The form SHALL show the demo email and ask for the password given in the Devpost private field. Sign-in SHALL use `POST /api/auth/login`.
4. THE SYSTEM SHALL refuse these actions for the demo account with `403 demo_account_restricted`: linking a provider, linking an email, changing email or password (register, reset), deleting the account, and `logout-all`. `logout` (this session only) SHALL be allowed.
5. THE demo account SHALL be reachable by password only. A magic-link request for its address SHALL answer the usual `202` and send nothing, its tokens SHALL be refused, and an OAuth profile with its address SHALL be refused like any other email collision.
6. THE database SHALL, as a second barrier, refuse to add an identity to the demo user, refuse to delete the demo user, and refuse to change its email, handle or password except during seeding.
7. ONCE per day at `DEMO_RESET_HOUR_UTC` (default 0 **[E]**), and on start-up when a scheduled reset was missed, THE SYSTEM SHALL reset the demo account's data: delete its finished submissions and every dependent row, and reset its progress counters. It SHALL keep the user and its sessions, and SHALL skip submissions that are still running.
8. THE reset SHALL touch no other user's data, SHALL be safe to run twice, and SHALL run on one instance at a time.
9. `GET /api/me` for the demo account SHALL report `is_demo: true` and all restricted capabilities as false.
10. Normal rate limits SHALL apply to the demo account.

### R17. Account deletion (Could; ACC-5)

**User story:** As a user, I want to delete my account and data, so that I control my information.

1. WHEN a signed-in user sends `DELETE /api/me {confirm_handle}` with their own handle, THE SYSTEM SHALL delete the account, its identities, sessions, tokens, flows, submissions and every dependent row, clear the cookie and answer `204`.
2. IF the handle does not match, THEN `400 confirmation_mismatch` and no change.
3. IF the account has a run that is still queued or running, THEN `409 runs_active` and no change.
4. THE demo account SHALL be refused (R16.4, R16.6).
5. THE SYSTEM SHALL also delete any unused email token for the account's address.

### R18. Data model, constraints and privileges (Must)

**User story:** As a developer, I want the database itself to refuse impossible states, so that a bug in application code cannot create them.

1. THE migrations SHALL create `users`, `identities`, `email_tokens`, `sessions` and `oauth_flows` with the columns in `design.md`.
2. `users.email` SHALL be nullable and unique (case-insensitive). THE database SHALL refuse any row where `email` is set and `email_verified_at` is not. THE database SHALL refuse a password hash on a user without an email. At most one user SHALL have `is_demo = true`.
3. `identities` SHALL be unique on (provider, provider_user_id) and on (user_id, provider), and SHALL carry no column used to match accounts.
4. `email_tokens` SHALL enforce the 15-minute lifetime and the purpose list. `oauth_flows` SHALL enforce the 10-minute lifetime and the shape of each purpose (`login` has a binding hash and no session; `link` has a session and user and no binding hash).
5. `sessions.user_id` SHALL be immutable and `absolute_expires_at` SHALL never exceed 90 days after creation.
6. EVERY foreign key, in any migration, that references `users(id)` or `submissions(id)` SHALL be `ON DELETE CASCADE`. A test SHALL fail when one is not.
7. WHEN `submissions` exists, a migration SHALL add `submissions.user_id → users(id) ON DELETE CASCADE` (the foreign key that `run-lifecycle` deferred to this spec).
8. THE role `counterspell_server` SHALL receive only the privileges it needs on the auth tables. THE role `counterspell_judge` and `PUBLIC` SHALL receive none. THE database functions that seed and reset the demo account SHALL be `SECURITY DEFINER` with a fixed `search_path` and executable only by `counterspell_server`.
9. THE migrations SHALL contain no secret and SHALL NOT seed the demo password.

### R19. Privacy and logging (Must)

**User story:** As the operator, I want auth events recorded without ever recording a secret, so that logs can be shared when debugging.

1. THE SYSTEM SHALL log auth events (sign-in success and failure, link, logout, logout-all, token issued, rate limited) as structured events containing the user id (a UUID), the method and a keyed hash of the IP, and no email address.
2. THE SYSTEM SHALL NEVER log: raw tokens, session values or keys, passwords, OAuth `code`, `state`, PKCE verifier, provider access tokens, client secrets, the Resend key, the `Authorization` header, or request bodies of auth endpoints.
3. THE HTTP trace layer SHALL log the request path without its query string for the OAuth callback and verify routes.
4. Error responses SHALL use the stable JSON shape of `api-conventions.md` (`{"code","message"}`) and SHALL NOT include SQL, provider responses, stack traces or secrets.

### R20. Web screens (Must for login; Should for the rest)

**User story:** As a visitor, I want clear, keyboard-friendly screens for signing in, confirming a link, and managing my account.

1. THE `/login` page SHALL offer a button for each enabled provider, an email form (magic link by default, password as an alternative), the demo link when enabled, and an error banner that maps each `error` code to a plain sentence.
2. THE `/auth/verify` page SHALL read the token from the URL fragment, remove it from the address bar at once, show a confirm button, call `POST /api/auth/email/verify` on click, and on success go to the validated same-origin `return_to`.
3. THE `/settings/account` page SHALL show the email, the linked providers, buttons to link GitHub, Google or an email (only where the capability allows), "sign out everywhere" and "delete account" (where implemented). For the demo account it SHALL show the actions as unavailable with a reason.
4. THE screens SHALL be fully usable by keyboard, with labelled inputs, visible focus and `aria-live` regions for errors and status, and SHALL not rely on colour alone.
5. THE screens SHALL use the semantic Steel Night tokens, no GSAP and no ad hoc colours. Server messages SHALL be rendered as plain text.
6. WHEN a protected API call returns `401`, THEN the web client SHALL go to `/login` with a sanitised `return_to`.
7. THE web client SHALL treat every client-side route guard as convenience only. The server decides.

### R21. Verification (Must)

**User story:** As the reviewer, I want proof that the security rules hold, not just code that looks right.

1. THE SYSTEM SHALL have property-based tests for the properties P1 to P22 in `design.md`, using a fake OAuth provider, a fake mailer and a manual clock. No automated test SHALL call GitHub, Google or Resend.
2. THE SYSTEM SHALL have database tests (named `pg_*`) for constraints, triggers, privileges and atomic consumption, run against a real Postgres in CI.
3. THE SYSTEM SHALL have hostile-input tests: replayed and concurrent token use, tampered or foreign `state`, wrong binding cookie, foreign session on a link flow, expired everything, cross-origin POST, open-redirect strings, enumeration probes, log capture.
4. THE definition of done SHALL include a manual smoke test with a real GitHub OAuth app, a real Google OAuth client and a real Resend sender on the VPS. That test is not automated.
5. THE CI checks of `repo-skeleton` (fmt, Clippy with `-D warnings`, tests, web check and build, `cargo deny check licenses`) SHALL stay green.

## Requirements analysis (Analyze Requirements pass)

Kiro's *Analyze Requirements* feature is interactive and was not available in this non-interactive CLI run. The same pass was done by hand, looking for conflicts with the documents, ambiguity, gaps and untestable wording. Each finding is resolved in the requirement named. **Re-run Analyze Requirements in the IDE before approving the design.**

| # | Finding | Kind | Resolution |
|---|---|---|---|
| 1 | `docs/system-design.md` §9 has `GET /api/auth/email/verify?token=`. A state-changing `GET` is consumed by mail scanners that pre-fetch links, so the user's single-use link is dead on arrival. | Conflict | Verify is `POST` behind a confirm button; the token travels in the URL fragment (R4.5, R5.3). Open question 1. |
| 2 | "Logout everywhere" is required by the brief but absent from §9. | Gap | `POST /api/auth/logout-all` (R11.2). |
| 3 | ACC-6 says the demo's data resets daily, but the `run-lifecycle` role matrix gives the server no `DELETE` on `submissions`. | Conflict | A `SECURITY DEFINER` function that can only reset the demo user (R16.7, R18.8). Open question 3. |
| 4 | ACC-2.1 says "30 days with rolling renewal"; rolling alone never expires an active session. | Ambiguity | Add a 90-day absolute limit (R10.3). Open question 4. |
| 5 | A `link_email` link opened in another browser, or by another signed-in user, could attach an address to the wrong account. | Gap | Consume only for a session of the token's user; mismatch does not burn the token (R9.3). |
| 6 | §4 says `email citext unique` but OAuth users may have no (verified) email. A stored unverified email would let someone squat an address. | Ambiguity | Nullable email; the database refuses an unverified stored email (R7.3, R18.2). |
| 7 | Resend's free plan allows 100 emails per day. One abuser could use it up and block every magic link. | Risk | Per-email, per-IP and a global daily cap; fail closed on Valkey outage (R4.8, R14.1, R14.4). |
| 8 | The demo account is shared by every judge. `logout-all` would sign the others out; a password change would lock them out. | Gap | Demo restrictions include `logout-all` and all credential changes (R16.4). |
| 9 | Registering with an unverified email as an account row lets an attacker squat the victim's address ("pre-hijacking"). | Risk | A pending registration lives in a token; no account until the mailbox is proven (R15.7). |
| 10 | ACC-1.4 says to tell the user to use their original method. That reveals that an email has an account. | Trade-off | Accepted only for the person who just proved control of that email at a provider (R7.2). Magic-link, register and reset answers stay uniform (R5.1, R15.7, R15.8). |
| 11 | §9 gives separate callbacks for sign-in and linking. GitHub OAuth apps accept one callback URL. | Conflict | One callback per provider; the flow row decides sign-in or link (R6, R8). Open question 2. |
| 12 | The CSRF rule must cover `run-lifecycle` routes too. | Gap | Global `/api` middleware (R13.5). |
| 13 | ACC-5 (delete account) is *Could* but the brief says the demo account "cannot delete itself". | Priority | The guard is *Must* (R16.4, R16.6); the endpoint is *Could* (R17). |
| 14 | Long-lived SSE streams would outlive a "logout everywhere". | Gap | A revalidation hook for streams (R11.6). |
| 15 | Wording like "securely", "quickly" was avoided; every criterion names a status code, a limit or an observable state, so each can be tested. | Testability | Numbers are marked [E] where they are defaults. |
| 16 | Tracing layers log the full URI by default; the OAuth callback URI carries `code` and `state`. | Risk | Path-only logging for those routes (R19.3). |

## Out of scope

- Unlinking a provider, changing an email address, adding a password to an OAuth-only account, multiple emails per account.
- Roles, admin screens, suspension (ADM-3), audit dashboards (ADM-2, ADM-4), profile editing.
- Multi-factor, passkeys, SSO, device lists, "remember this device".
- Any change to run, SSE, catalogue, hints or dashboard behaviour, except that they now see a real signed-in user.
- Java, C++, payments, FSRS, the break room.
