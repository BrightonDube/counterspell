# D.8 Spec 6: `auth` (Feature, Requirements-First, Analyze + PBT)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Create a feature spec named auth (Requirements-First). Context: #[[file:docs/system-design.md]] §1, §4 users/sessions tables, §9 auth endpoints; email via Resend. GitHub + Google OAuth (state + PKCE) and email magic link (single-use, 15-min expiry), cookie sessions (HttpOnly, Secure, SameSite=Lax), logout everywhere. Keep all three sign-ins, but NO automatic account linking by matching email: linking happens only when a signed-in user starts it, via OAuth with a link-intent state bound to the session or a verification email (#[[file:docs/requirements.md]] ACC-1, ACC-3). Add the judge demo account path (ACC-6): seeded demo user, sign-in link on the login page, can't link providers, change credentials or delete itself, daily reset. Write EARS criteria that yield properties (token single-use, expiry, a session never reads another user's data, an OAuth email match never links or signs in to another account). Run Analyze Requirements. Never touch .env; read secrets from env vars only.
```
