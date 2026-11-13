# Auth blockers

Run: auth implement, attempt 1 (13 Nov 2026).

## T01: licence decision needed for reqwest and oauth2 (BLOCKED, Brighton)

- **Problem:** `reqwest 0.12` with the `rustls-tls` feature pulls in `webpki-roots 1.0.12`, which is licensed `CDLA-Permissive-2.0`. `cargo deny check licenses` rejects it, because `deny.toml` does not allow that licence. `oauth2 5.0` also depends on `reqwest`, so it fails the same way.
- **What I did:** added every other T01 crate (see `THIRD-PARTY-NOTICES.md`, "Auth dependencies"). The licence check passes without `reqwest` and `oauth2`. I did not edit `deny.toml`, because the spec says to ask first.
- **Unblock options (Brighton to pick):**
  1. Allow `CDLA-Permissive-2.0` in `deny.toml`. This is a data-only licence (root certificates), so the risk is low.
  2. Change the `reqwest` feature from `rustls-tls` to `rustls-tls-native-roots`. That drops `webpki-roots` and keeps the licence list unchanged, but it uses the operating system's certificate store.
- **Tasks affected until this is decided:** T14 (Resend sender), T18 (OAuth provider trait and PKCE via `oauth2`), T19 (GitHub), T20 (Google), T21 (OAuth sign-in), T22 (link provider), T23 (link email, depends on T14 and T17), T17 (magic-link handlers, depends on T14), T25 (register and reset, depends on T14), and T33 (wiring). Tasks that do not need these crates continue.
- **Assumed choice while blocked:** option 2 would be my choice, because it keeps the licence list unchanged. Not applied.
