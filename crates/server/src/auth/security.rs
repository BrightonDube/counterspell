//! Account credentials and the gap between a successful first factor and a full session.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, Extension, State},
    http::HeaderMap,
    Json,
};
use counterspell_common::ids::UserId;
use ring::hmac;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Postgres, Transaction};
use time::Duration;
use url::Url;
use uuid::Uuid;
use webauthn_rs::{Webauthn, WebauthnBuilder};

use super::{
    capabilities::{Capabilities, Capability},
    config::Redacted,
    cookies::{read_cookie, CookieSlot},
    errors::{database, AuthError},
    extract::AuthUser,
    limits::{Action, Scope},
    passkeys,
    return_to::sanitize_return_to,
    session::AuthMethod,
    tokens::RawToken,
    totp, AuthState,
};
use crate::client_ip::client_ip;

pub(crate) const STEP_UP_PATH: &str = "/login?step=2fa";
const CHALLENGE_TTL: Duration = Duration::minutes(5);
const RECENT_SIGN_IN: Duration = Duration::minutes(15);

pub(crate) type AuthJson<T> = Result<Json<T>, JsonRejection>;

pub(crate) fn decode_json<T>(input: AuthJson<T>) -> Result<T, AuthError> {
    input
        .map(|Json(value)| value)
        .map_err(|_| AuthError::InvalidJson)
}

#[derive(Clone)]
pub(crate) struct SecuritySettings {
    credential_key: Arc<Redacted<[u8; 32]>>,
    webauthn: Option<Arc<Webauthn>>,
}

impl SecuritySettings {
    pub(crate) fn new(origin: &str, secret: &[u8]) -> Self {
        let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
        let tag = hmac::sign(&key, b"counterspell/auth/credential-encryption/v1");
        let mut credential_key = [0; 32];
        credential_key.copy_from_slice(tag.as_ref());
        let webauthn = Url::parse(origin).ok().and_then(|url| {
            let rp = url.domain()?;
            if url.scheme() != "https" && !(url.scheme() == "http" && rp == "localhost") {
                return None;
            }
            WebauthnBuilder::new(rp, &url)
                .ok()?
                .rp_name("Counterspell")
                .build()
                .ok()
                .map(Arc::new)
        });
        Self {
            credential_key: Arc::new(Redacted::new(credential_key)),
            webauthn,
        }
    }
}

pub(crate) fn credential_key(state: &AuthState) -> Result<&[u8; 32], AuthError> {
    state
        .security
        .as_ref()
        .map(|settings| settings.credential_key.expose())
        .ok_or(AuthError::SecurityUnavailable)
}

pub(crate) fn webauthn(state: &AuthState) -> Result<&Webauthn, AuthError> {
    state
        .security
        .as_ref()
        .and_then(|settings| settings.webauthn.as_deref())
        .ok_or(AuthError::SecurityUnavailable)
}

pub(crate) async fn limit(state: &AuthState, key: &str) -> Result<(), AuthError> {
    state.limits.hit(Action::Security, Scope::User, key).await
}

pub(crate) async fn require_recent(state: &AuthState, actor: &AuthUser) -> Result<(), AuthError> {
    let mut connection = state.pool.acquire().await.map_err(database)?;
    require_recent_in(&mut connection, state, actor).await
}

pub(crate) async fn require_recent_in(
    connection: &mut PgConnection,
    state: &AuthState,
    actor: &AuthUser,
) -> Result<(), AuthError> {
    Capabilities::for_user(actor.is_demo).require(Capability::ChangeCredentials)?;
    let now = state.clock.now();
    let recent: Option<String> = sqlx::query_scalar(
        "SELECT id FROM sessions
          WHERE id = $1 AND user_id = $2 AND created_at >= $3
            AND expires_at > $4 AND absolute_expires_at > $4 FOR UPDATE",
    )
    .bind(actor.key.as_str())
    .bind(actor.user_id.as_uuid())
    .bind(now - RECENT_SIGN_IN)
    .bind(now)
    .fetch_optional(connection)
    .await
    .map_err(database)?;
    if recent.is_none() {
        return Err(AuthError::RecentSignInRequired);
    }
    Ok(())
}

pub(crate) async fn lock_user(
    connection: &mut PgConnection,
    user: UserId,
) -> Result<(), AuthError> {
    let found: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(user.as_uuid())
        .fetch_optional(connection)
        .await
        .map_err(database)?;
    found.ok_or(AuthError::Unauthenticated)?;
    Ok(())
}

pub(crate) async fn requires_factor(
    connection: &mut PgConnection,
    user: UserId,
) -> Result<bool, AuthError> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM auth_totp WHERE user_id = $1)")
        .bind(user.as_uuid())
        .fetch_one(connection)
        .await
        .map_err(database)
}

fn token_hash(value: &str) -> Vec<u8> {
    Sha256::digest(value.as_bytes()).to_vec()
}

/// Completes the first factor. A pending challenge grants no authenticated session.
pub(crate) async fn complete_primary(
    state: &AuthState,
    slot: &CookieSlot,
    user: UserId,
    method: AuthMethod,
    return_to: &str,
) -> Result<bool, AuthError> {
    let mut tx = state.pool.begin().await.map_err(database)?;
    lock_user(&mut tx, user).await?;
    complete_primary_locked(state, slot, tx, user, method, return_to).await
}

/// Finishes the sign-in in the transaction that locked the user and checked the first factor.
pub(crate) async fn complete_primary_locked(
    state: &AuthState,
    slot: &CookieSlot,
    mut tx: Transaction<'_, Postgres>,
    user: UserId,
    method: AuthMethod,
    return_to: &str,
) -> Result<bool, AuthError> {
    if requires_factor(&mut tx, user).await? {
        credential_key(state)?;
        let challenge = RawToken::generate()?;
        sqlx::query(
            "INSERT INTO auth_step_up (token_hash, user_id, auth_method, return_to, expires_at)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(token_hash(challenge.expose()))
        .bind(user.as_uuid())
        .bind(method.as_str())
        .bind(sanitize_return_to(Some(return_to)))
        .bind(state.clock.now() + CHALLENGE_TTL)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        tx.commit().await.map_err(database)?;
        slot.push(state.cookies.clear_session()?);
        slot.push(state.cookies.step_up_value(challenge.expose())?);
        return Ok(true);
    }
    let created = state
        .sessions
        .create_in(&mut tx, user, method, false)
        .await?;
    tx.commit().await.map_err(database)?;
    slot.push(state.cookies.session_value(created.cookie_value.expose())?);
    slot.push(state.cookies.clear_step_up()?);
    Ok(false)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerifyRequest {
    code: String,
}

#[derive(Serialize)]
pub(crate) struct Verified {
    return_to: String,
}

pub(crate) async fn verify(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: AuthJson<VerifyRequest>,
) -> Result<Json<Verified>, AuthError> {
    let request: VerifyRequest = decode_json(body)?;
    let key = credential_key(&state)?;
    let raw = read_cookie(&headers, state.cookies.step_up_name())
        .ok_or(AuthError::SecurityChallengeExpired)?;
    let hash = token_hash(raw);
    let ip = client_ip(peer.ip(), &headers, &state.trusted);
    state
        .limits
        .hit(Action::Security, Scope::Ip, &format!("step-up:{ip}"))
        .await?;
    let now = state.clock.now();
    let user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM auth_step_up WHERE token_hash = $1 AND expires_at > $2",
    )
    .bind(&hash)
    .bind(now)
    .fetch_optional(&state.pool)
    .await
    .map_err(database)?
    .ok_or(AuthError::SecurityChallengeExpired)?;
    limit(&state, &format!("step-up:{user_id}")).await?;
    let user = UserId::from_uuid(user_id);
    let mut tx = state.pool.begin().await.map_err(database)?;
    lock_user(&mut tx, user).await?;
    let (method, target): (String, String) = sqlx::query_as(
        "SELECT auth_method, return_to FROM auth_step_up
          WHERE token_hash = $1 AND user_id = $2 AND expires_at > $3 FOR UPDATE",
    )
    .bind(&hash)
    .bind(user_id)
    .bind(state.clock.now())
    .fetch_optional(&mut *tx)
    .await
    .map_err(database)?
    .ok_or(AuthError::SecurityChallengeExpired)?;
    let now = state.clock.now();
    let verified = totp::consume_code(&mut tx, key, user, &request.code, now)
        .await?
        .ok_or(AuthError::InvalidSecurityCode)?;
    sqlx::query("DELETE FROM auth_step_up WHERE token_hash = $1")
        .bind(&hash)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    let method = AuthMethod::from_stored(&method)?;
    let created = state
        .sessions
        .create_in(&mut tx, user, method, true)
        .await?;
    if verified == totp::VerifiedCode::Recovery {
        sqlx::query("UPDATE sessions SET authenticator_recovery_until = $1 WHERE id = $2")
            .bind(now + totp::RECOVERY_TTL)
            .bind(created.key.as_str())
            .execute(&mut *tx)
            .await
            .map_err(database)?;
    }
    tx.commit().await.map_err(database)?;
    slot.push(state.cookies.session_value(created.cookie_value.expose())?);
    slot.push(state.cookies.clear_step_up()?);
    Ok(Json(Verified {
        return_to: sanitize_return_to(Some(&target)),
    }))
}

pub(crate) async fn end_pending(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
) -> Result<(), AuthError> {
    if let Some(raw) = read_cookie(headers, state.cookies.step_up_name()) {
        sqlx::query("DELETE FROM auth_step_up WHERE token_hash = $1")
            .bind(token_hash(raw))
            .execute(&state.pool)
            .await
            .map_err(database)?;
        slot.push(state.cookies.clear_step_up()?);
    }
    Ok(())
}

#[derive(Serialize)]
pub(crate) struct SecuritySummary {
    totp_enabled: bool,
    can_replace_authenticator: bool,
    passkeys: Vec<passkeys::PasskeySummary>,
}

pub(crate) async fn summary(
    State(state): State<AuthState>,
    actor: AuthUser,
) -> Result<Json<SecuritySummary>, AuthError> {
    let mut connection = state.pool.acquire().await.map_err(database)?;
    let totp_enabled = requires_factor(&mut connection, actor.user_id).await?;
    let can_replace_authenticator =
        totp_enabled && totp::can_replace(&mut connection, &actor, state.clock.now()).await?;
    drop(connection);
    let passkeys = passkeys::list(&state.pool, actor.user_id).await?;
    Ok(Json(SecuritySummary {
        totp_enabled,
        can_replace_authenticator,
        passkeys,
    }))
}
