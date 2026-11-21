//! Authenticator enrollment and single-use TOTP and recovery-code verification.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use counterspell_common::ids::UserId;
use data_encoding::BASE32_NOPAD;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN},
    hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use subtle::ConstantTimeEq;
use time::{Duration, OffsetDateTime};

use super::{
    errors::{database, AuthError},
    extract::AuthUser,
    security,
    session::SessionKey,
    tokens::TokenHash,
    AuthState,
};

const SETUP_TTL: Duration = Duration::minutes(10);
pub(crate) const RECOVERY_TTL: Duration = Duration::minutes(10);
const STEP_SECONDS: i64 = 30;
const SECRET_BYTES: usize = 20;
const RECOVERY_COUNT: usize = 10;
const RECOVERY_BYTES: usize = 15;
const RECOVERY_LENGTH: usize = RECOVERY_BYTES * 8 / 5;
const RECOVERY_GROUP: usize = 4;
const DISPLAYED_RECOVERY_LENGTH: usize = RECOVERY_LENGTH + (RECOVERY_LENGTH - 1) / RECOVERY_GROUP;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum VerifiedCode {
    Authenticator,
    Recovery,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EmptyBody {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CodeBody {
    code: String,
}

#[derive(Serialize)]
struct SetupResponse {
    secret: String,
    uri: String,
}

#[derive(Serialize)]
struct EnableResponse {
    recovery_codes: Vec<String>,
}

/// Generates a short-lived authenticator secret for the current session.
pub(crate) async fn start(
    State(state): State<AuthState>,
    actor: AuthUser,
    body: security::AuthJson<EmptyBody>,
) -> Result<impl IntoResponse, AuthError> {
    let _: EmptyBody = security::decode_json(body)?;
    authorize(&state, &actor).await?;
    let key = security::credential_key(&state)?;
    let mut tx = state.pool.begin().await.map_err(database)?;
    let handle = lock_actor(&mut tx, &actor, state.clock.now()).await?;
    permit_setup(&mut tx, &actor, state.clock.now()).await?;

    let secret = random::<SECRET_BYTES>()?;
    let scope = setup_aad(actor.user_id, &actor.key);
    let encrypted = encrypt(key, scope.as_bytes(), &secret)?;
    sqlx::query(
        "INSERT INTO auth_totp_setup (user_id, secret, expires_at) VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE SET secret = EXCLUDED.secret, expires_at = EXCLUDED.expires_at",
    )
    .bind(actor.user_id.as_uuid())
    .bind(encrypted)
    .bind(state.clock.now() + SETUP_TTL)
    .execute(&mut *tx)
    .await
    .map_err(database)?;
    tx.commit().await.map_err(database)?;

    let secret = BASE32_NOPAD.encode(&secret);
    let label = utf8_percent_encode(&handle, NON_ALPHANUMERIC);
    let uri = format!(
        "otpauth://totp/Counterspell:{label}?secret={secret}&issuer=Counterspell&algorithm=SHA1&digits=6&period={STEP_SECONDS}"
    );
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(SetupResponse { secret, uri }),
    ))
}

/// Verifies the first authenticator code and returns one set of recovery codes.
pub(crate) async fn enable(
    State(state): State<AuthState>,
    actor: AuthUser,
    body: security::AuthJson<CodeBody>,
) -> Result<impl IntoResponse, AuthError> {
    let CodeBody { code } = security::decode_json(body)?;
    authorize(&state, &actor).await?;
    let key = security::credential_key(&state)?;
    let mut tx = state.pool.begin().await.map_err(database)?;
    lock_actor(&mut tx, &actor, state.clock.now()).await?;
    permit_setup(&mut tx, &actor, state.clock.now()).await?;

    let now = state.clock.now();
    // Verification failures roll back the deletion, so a typo can be retried.
    let mut pending: Vec<u8> = sqlx::query_scalar(
        "DELETE FROM auth_totp_setup WHERE user_id = $1 AND expires_at > $2 RETURNING secret",
    )
    .bind(actor.user_id.as_uuid())
    .bind(now)
    .fetch_optional(&mut *tx)
    .await
    .map_err(database)?
    .ok_or(AuthError::SecurityChallengeExpired)?;
    let scope = setup_aad(actor.user_id, &actor.key);
    let secret =
        decrypt(key, scope.as_bytes(), &mut pending).ok_or(AuthError::SecurityChallengeExpired)?;
    let step = valid_step(secret, code.trim(), -1, now.unix_timestamp())
        .ok_or(AuthError::InvalidSecurityCode)?;
    let scope = active_aad(actor.user_id);
    let encrypted = encrypt(key, scope.as_bytes(), secret)?;
    // A replacement rotates the authenticator and all of its recovery codes together.
    sqlx::query("DELETE FROM auth_totp WHERE user_id = $1")
        .bind(actor.user_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    sqlx::query(
        "INSERT INTO auth_totp (user_id, secret, last_step, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(actor.user_id.as_uuid())
    .bind(encrypted)
    .bind(step)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(database)?;

    let mut recovery_codes = Vec::with_capacity(RECOVERY_COUNT);
    for _ in 0..RECOVERY_COUNT {
        let mut code = BASE32_NOPAD.encode(&random::<RECOVERY_BYTES>()?);
        let hash = TokenHash::of(&code);
        sqlx::query("INSERT INTO auth_recovery_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(actor.user_id.as_uuid())
            .bind(hash.as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(database)?;
        for index in (RECOVERY_GROUP..code.len()).step_by(RECOVERY_GROUP).rev() {
            code.insert(index, '-');
        }
        recovery_codes.push(code);
    }

    sqlx::query(
        "UPDATE sessions SET second_factor_verified = true, authenticator_recovery_until = NULL
          WHERE user_id = $1 AND id = $2",
    )
    .bind(actor.user_id.as_uuid())
    .bind(actor.key.as_str())
    .execute(&mut *tx)
    .await
    .map_err(database)?;
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
        .bind(actor.user_id.as_uuid())
        .bind(actor.key.as_str())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    clear_step_up(&mut tx, actor.user_id).await?;
    tx.commit().await.map_err(database)?;
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(EnableResponse { recovery_codes }),
    ))
}

/// Removes the authenticator after verifying a fresh authenticator or recovery code.
pub(crate) async fn disable(
    State(state): State<AuthState>,
    actor: AuthUser,
    body: security::AuthJson<CodeBody>,
) -> Result<StatusCode, AuthError> {
    let CodeBody { code } = security::decode_json(body)?;
    authorize(&state, &actor).await?;
    let key = security::credential_key(&state)?;
    let mut tx = state.pool.begin().await.map_err(database)?;
    lock_actor(&mut tx, &actor, state.clock.now()).await?;
    if consume_code(&mut tx, key, actor.user_id, &code, state.clock.now())
        .await?
        .is_none()
    {
        return Err(AuthError::InvalidSecurityCode);
    }
    sqlx::query("DELETE FROM auth_totp WHERE user_id = $1")
        .bind(actor.user_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    sqlx::query("DELETE FROM auth_totp_setup WHERE user_id = $1")
        .bind(actor.user_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    sqlx::query(
        "UPDATE sessions SET authenticator_recovery_until = NULL
          WHERE user_id = $1 AND authenticator_recovery_until IS NOT NULL",
    )
    .bind(actor.user_id.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(database)?;
    clear_step_up(&mut tx, actor.user_id).await?;
    tx.commit().await.map_err(database)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Consumes a credential inside the caller's transaction, after it locks the owning user row.
pub(crate) async fn consume_code(
    tx: &mut PgConnection,
    key: &[u8; 32],
    user_id: UserId,
    code: &str,
    now: OffsetDateTime,
) -> Result<Option<VerifiedCode>, AuthError> {
    let code = code.trim();
    if is_totp(code) {
        let row: Option<(Vec<u8>, i64)> =
            sqlx::query_as("SELECT secret, last_step FROM auth_totp WHERE user_id = $1 FOR UPDATE")
                .bind(user_id.as_uuid())
                .fetch_optional(&mut *tx)
                .await
                .map_err(database)?;
        let Some((mut encrypted, last_step)) = row else {
            return Ok(None);
        };
        let scope = active_aad(user_id);
        let secret =
            decrypt(key, scope.as_bytes(), &mut encrypted).ok_or(AuthError::SecurityUnavailable)?;
        let Some(step) = valid_step(secret, code, last_step, now.unix_timestamp()) else {
            return Ok(None);
        };
        let result = sqlx::query(
            "UPDATE auth_totp SET last_step = $2 WHERE user_id = $1 AND last_step < $2",
        )
        .bind(user_id.as_uuid())
        .bind(step)
        .execute(&mut *tx)
        .await
        .map_err(database)?;
        return Ok((result.rows_affected() == 1).then_some(VerifiedCode::Authenticator));
    }

    let Some(hash) = recovery_hash(code) else {
        return Ok(None);
    };
    let result =
        sqlx::query("DELETE FROM auth_recovery_codes WHERE user_id = $1 AND code_hash = $2")
            .bind(user_id.as_uuid())
            .bind(hash.as_bytes().as_slice())
            .execute(&mut *tx)
            .await
            .map_err(database)?;
    Ok((result.rows_affected() == 1).then_some(VerifiedCode::Recovery))
}

async fn authorize(state: &AuthState, actor: &AuthUser) -> Result<(), AuthError> {
    security::require_recent(state, actor).await?;
    security::limit(state, &format!("totp:{}", actor.user_id)).await
}

async fn lock_actor(
    tx: &mut PgConnection,
    actor: &AuthUser,
    now: OffsetDateTime,
) -> Result<String, AuthError> {
    let handle = sqlx::query_scalar("SELECT handle::text FROM users WHERE id = $1 FOR UPDATE")
        .bind(actor.user_id.as_uuid())
        .fetch_optional(&mut *tx)
        .await
        .map_err(database)?
        .ok_or(AuthError::Unauthenticated)?;
    // A concurrent revocation may have removed the session after the extractor read it.
    sqlx::query_scalar::<_, String>(
        "SELECT id FROM sessions WHERE user_id = $1 AND id = $2 AND expires_at > $3 FOR UPDATE",
    )
    .bind(actor.user_id.as_uuid())
    .bind(actor.key.as_str())
    .bind(now)
    .fetch_optional(tx)
    .await
    .map_err(database)?
    .ok_or(AuthError::Unauthenticated)?;
    Ok(handle)
}

pub(crate) async fn can_replace(
    connection: &mut PgConnection,
    actor: &AuthUser,
    now: OffsetDateTime,
) -> Result<bool, AuthError> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sessions
          WHERE id = $1 AND user_id = $2 AND authenticator_recovery_until > $3)",
    )
    .bind(actor.key.as_str())
    .bind(actor.user_id.as_uuid())
    .bind(now)
    .fetch_one(connection)
    .await
    .map_err(database)
}

async fn permit_setup(
    tx: &mut PgConnection,
    actor: &AuthUser,
    now: OffsetDateTime,
) -> Result<(), AuthError> {
    if security::requires_factor(tx, actor.user_id).await? && !can_replace(tx, actor, now).await? {
        return Err(AuthError::SecurityConflict);
    }
    Ok(())
}

async fn clear_step_up(tx: &mut PgConnection, user_id: UserId) -> Result<(), AuthError> {
    sqlx::query("DELETE FROM auth_step_up WHERE user_id = $1")
        .bind(user_id.as_uuid())
        .execute(tx)
        .await
        .map_err(database)?;
    Ok(())
}

fn random<const N: usize>() -> Result<[u8; N], AuthError> {
    let mut bytes = [0; N];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| AuthError::Internal)?;
    Ok(bytes)
}

fn active_aad(user_id: UserId) -> String {
    format!("counterspell/totp/v1/active/{user_id}")
}

fn setup_aad(user_id: UserId, session: &SessionKey) -> String {
    format!("counterspell/totp/v1/setup/{user_id}/{}", session.as_str())
}

fn encrypt(key: &[u8; 32], aad: &[u8], secret: &[u8]) -> Result<Vec<u8>, AuthError> {
    let key =
        LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).map_err(|_| AuthError::Internal)?);
    let nonce = random::<NONCE_LEN>()?;
    let mut ciphertext = Vec::with_capacity(NONCE_LEN + secret.len() + AES_256_GCM.tag_len());
    ciphertext.extend_from_slice(&nonce);
    ciphertext.extend_from_slice(secret);
    let tag = key
        .seal_in_place_separate_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(aad),
            &mut ciphertext[NONCE_LEN..],
        )
        .map_err(|_| AuthError::Internal)?;
    ciphertext.extend_from_slice(tag.as_ref());
    Ok(ciphertext)
}

fn decrypt<'a>(key: &[u8; 32], aad: &[u8], encrypted: &'a mut [u8]) -> Option<&'a [u8]> {
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).ok()?);
    let (nonce, ciphertext) = encrypted.split_at_mut_checked(NONCE_LEN)?;
    key.open_in_place(
        Nonce::try_assume_unique_for_key(nonce).ok()?,
        Aad::from(aad),
        ciphertext,
    )
    .ok()
    .map(|secret| &*secret)
}

fn is_totp(code: &str) -> bool {
    code.len() == 6 && code.bytes().all(|byte| byte.is_ascii_digit())
}

fn recovery_hash(code: &str) -> Option<TokenHash> {
    if !matches!(code.len(), RECOVERY_LENGTH | DISPLAYED_RECOVERY_LENGTH) {
        return None;
    }
    let normalized = code.replace('-', "").to_ascii_uppercase();
    if normalized.len() != RECOVERY_LENGTH || BASE32_NOPAD.decode(normalized.as_bytes()).is_err() {
        return None;
    }
    Some(TokenHash::of(&normalized))
}

fn valid_step(secret: &[u8], code: &str, last_step: i64, unix_seconds: i64) -> Option<i64> {
    if !is_totp(code) {
        return None;
    }
    let current = unix_seconds.div_euclid(STEP_SECONDS);
    // Prefer the latest step if two accepted timestamps happen to produce the same code.
    ((current - 1).max(0)..=current + 1).rev().find(|step| {
        *step > last_step && bool::from(totp_code(secret, *step).as_bytes().ct_eq(code.as_bytes()))
    })
}

fn totp_code(secret: &[u8], step: i64) -> String {
    let key = hmac::Key::new(HMAC_SHA1_FOR_LEGACY_USE_ONLY, secret);
    let tag = hmac::sign(&key, &step.to_be_bytes());
    let bytes = tag.as_ref();
    let offset = usize::from(bytes[19] & 0xf);
    let value = u32::from_be_bytes([
        bytes[offset] & 0x7f,
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]);
    format!("{:06}", value % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_SECRET: &[u8] = b"12345678901234567890";

    #[test]
    fn matches_rfc_6238_sha1_vectors_at_six_digits() {
        for (timestamp, code) in [
            (59, "287082"),
            (1_111_111_109, "081804"),
            (1_111_111_111, "050471"),
            (1_234_567_890, "005924"),
            (2_000_000_000, "279037"),
            (20_000_000_000, "353130"),
        ] {
            assert_eq!(totp_code(RFC_SECRET, timestamp / 30), code);
        }
    }

    #[test]
    fn rejects_consumed_and_out_of_window_codes() {
        assert_eq!(valid_step(RFC_SECRET, "287082", 0, 59), Some(1));
        assert_eq!(valid_step(RFC_SECRET, "287082", 1, 59), None);
        assert_eq!(valid_step(RFC_SECRET, "287082", 2, 59), None);
        assert_eq!(valid_step(RFC_SECRET, "287082", 0, 29), Some(1));
        assert_eq!(valid_step(RFC_SECRET, "287082", 0, 89), Some(1));
        assert_eq!(valid_step(RFC_SECRET, "287082", 0, 90), None);
    }

    #[test]
    fn recovery_code_hash_accepts_pasted_and_typed_forms() {
        let stored = TokenHash::of("ABCDEFGHIJKLMNOPQRSTUVWX");
        assert_eq!(recovery_hash("abcd-EFgH-ijkl-mnop-qrsT-Uvwx"), Some(stored));
        assert_eq!(recovery_hash("abcdefghijklmnopqrstuvwx"), Some(stored));
        assert_eq!(recovery_hash("abcd-EFgH-ijkl-mnop-qrsT-Uvw0"), None);
        assert_eq!(recovery_hash("abcd-EFgH-ijkl-mnop-qrsT"), None);
    }

    #[test]
    fn encrypted_secret_cannot_move_between_accounts_or_sessions() {
        let key = &[7; 32];
        let alice = UserId::from_uuid(uuid::Uuid::from_u128(1));
        let bob = UserId::from_uuid(uuid::Uuid::from_u128(2));
        let session = SessionKey::of("session-a");
        let other_session = SessionKey::of("session-b");
        let scope = setup_aad(alice, &session);
        let ciphertext = encrypt(key, scope.as_bytes(), RFC_SECRET).unwrap();
        assert_eq!(
            decrypt(key, scope.as_bytes(), &mut ciphertext.clone()),
            Some(RFC_SECRET)
        );
        assert!(decrypt(
            key,
            setup_aad(bob, &session).as_bytes(),
            &mut ciphertext.clone()
        )
        .is_none());
        assert!(decrypt(
            key,
            setup_aad(alice, &other_session).as_bytes(),
            &mut ciphertext.clone()
        )
        .is_none());
        assert!(decrypt(key, active_aad(alice).as_bytes(), &mut ciphertext.clone()).is_none());
        let mut tampered = ciphertext;
        tampered[12] ^= 1;
        assert!(decrypt(key, scope.as_bytes(), &mut tampered).is_none());
    }
}
