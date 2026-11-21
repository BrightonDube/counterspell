//! Discoverable `WebAuthn` passkeys with session-bound enrollment and browser-bound sign-in.

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Extension, Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use counterspell_common::ids::UserId;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json as DbJson, PgPool};
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
use uuid::Uuid;
use webauthn_rs::prelude::{
    CreationChallengeResponse, CredentialID, DiscoverableAuthentication, DiscoverableKey, Passkey,
    PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential, Webauthn,
    WebauthnResult,
};
use webauthn_rs_proto::{
    PublicKeyCredentialCreationOptions, PublicKeyCredentialRequestOptions, ResidentKeyRequirement,
    UserVerificationPolicy,
};

use crate::client_ip::client_ip;

use super::{
    cookies::{read_cookie, CookieSlot},
    errors::{database, AuthError},
    extract::AuthUser,
    return_to::{sanitize_return_to, DEFAULT_RETURN_TO},
    security,
    session::AuthMethod,
    tokens::{RawToken, TokenHash},
    AuthState,
};

const CHALLENGE_TTL: Duration = Duration::minutes(5);
const MAX_PASSKEYS: i64 = 8;
const MAX_CREDENTIAL_ID_BYTES: usize = 1023;

#[derive(Serialize)]
pub(crate) struct PasskeySummary {
    id: Uuid,
    name: String,
    created_at: String,
}

pub(crate) async fn list(pool: &PgPool, user_id: UserId) -> Result<Vec<PasskeySummary>, AuthError> {
    let rows: Vec<(Uuid, String, OffsetDateTime)> = sqlx::query_as(
        "SELECT id, name, created_at FROM auth_passkeys WHERE user_id = $1 ORDER BY created_at, id",
    )
    .bind(user_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(database)?;
    rows.into_iter()
        .map(|(id, name, created_at)| {
            Ok(PasskeySummary {
                id,
                name,
                created_at: created_at
                    .format(&Rfc3339)
                    .map_err(|_| AuthError::Internal)?,
            })
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegisterStart {
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoginStart {
    return_to: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Finish<C> {
    flow_id: Uuid,
    credential: C,
}

#[derive(Serialize)]
pub(crate) struct Start<C> {
    flow_id: Uuid,
    #[serde(rename = "publicKey")]
    public_key: C,
}

#[derive(Serialize)]
pub(crate) struct SignedIn {
    return_to: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    two_factor_required: bool,
}

pub(crate) async fn register_start(
    State(state): State<AuthState>,
    actor: AuthUser,
    body: security::AuthJson<RegisterStart>,
) -> Result<Json<Start<PublicKeyCredentialCreationOptions>>, AuthError> {
    security::require_recent(&state, &actor).await?;
    let webauthn = security::webauthn(&state)?;
    security::limit(&state, &format!("passkey:register:{}", actor.user_id)).await?;
    let input: RegisterStart = security::decode_json(body)?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err(AuthError::InvalidJson);
    }

    let user_id = actor.user_id.as_uuid();
    let (handle, display_name): (String, String) =
        sqlx::query_as("SELECT handle::text, display_name FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(database)?
            .ok_or(AuthError::Unauthenticated)?;
    let credentials: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT credential_id FROM auth_passkeys WHERE user_id = $1 LIMIT $2")
            .bind(user_id)
            .bind(MAX_PASSKEYS)
            .fetch_all(&state.pool)
            .await
            .map_err(database)?;
    if i64::try_from(credentials.len()).unwrap_or(i64::MAX) >= MAX_PASSKEYS {
        return Err(AuthError::PasskeyLimit);
    }
    let excluded = credentials.into_iter().map(CredentialID::from).collect();
    let (options, flow) =
        start_registration(webauthn, user_id, &handle, &display_name, Some(excluded))
            .map_err(|_| AuthError::PasskeyFailed)?;

    let mut tx = state.pool.begin().await.map_err(database)?;
    security::lock_user(&mut tx, actor.user_id).await?;
    security::require_recent_in(&mut tx, &state, &actor).await?;
    let flow_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth_passkey_flows (id, user_id, session_id, kind, state, name, return_to, expires_at) \
         VALUES ($1, $2, $3, 'register', $4, $5, $6, $7)",
    )
    .bind(flow_id)
    .bind(user_id)
    .bind(actor.key.as_str())
    .bind(DbJson(flow))
    .bind(name)
    .bind(DEFAULT_RETURN_TO)
    .bind(state.clock.now() + CHALLENGE_TTL)
    .execute(&mut *tx)
    .await
    .map_err(database)?;
    tx.commit().await.map_err(database)?;

    Ok(Json(Start {
        flow_id,
        public_key: options.public_key,
    }))
}

fn start_registration(
    webauthn: &Webauthn,
    id: Uuid,
    handle: &str,
    display_name: &str,
    excluded: Option<Vec<CredentialID>>,
) -> WebauthnResult<(CreationChallengeResponse, PasskeyRegistration)> {
    let (mut options, flow) =
        webauthn.start_passkey_registration(id, handle, display_name, excluded)?;
    // All passkeys here must work without an account hint at sign-in.
    let selection = options
        .public_key
        .authenticator_selection
        .get_or_insert_with(Default::default);
    selection.resident_key = Some(ResidentKeyRequirement::Required);
    selection.require_resident_key = true;
    selection.user_verification = UserVerificationPolicy::Required;
    Ok((options, flow))
}

pub(crate) async fn register_finish(
    State(state): State<AuthState>,
    actor: AuthUser,
    body: security::AuthJson<Finish<RegisterPublicKeyCredential>>,
) -> Result<StatusCode, AuthError> {
    let mut tx = state.pool.begin().await.map_err(database)?;
    security::lock_user(&mut tx, actor.user_id).await?;
    security::require_recent_in(&mut tx, &state, &actor).await?;
    let webauthn = security::webauthn(&state)?;
    let input: Finish<RegisterPublicKeyCredential> = security::decode_json(body)?;
    let user_id = actor.user_id.as_uuid();
    let (DbJson(flow), name): (DbJson<PasskeyRegistration>, String) = sqlx::query_as(
        "DELETE FROM auth_passkey_flows \
         WHERE id = $1 AND user_id = $2 AND session_id = $3 AND kind = 'register' AND expires_at > $4 \
         RETURNING state, name",
    )
    .bind(input.flow_id)
    .bind(user_id)
    .bind(actor.key.as_str())
    .bind(state.clock.now())
    .fetch_optional(&mut *tx)
    .await
    .map_err(database)?
    .ok_or(AuthError::SecurityChallengeExpired)?;

    let Some(passkey) = verified_registration(webauthn, &input.credential, &flow) else {
        // Commit the consumed challenge even when verification fails.
        tx.commit().await.map_err(database)?;
        return Err(AuthError::PasskeyFailed);
    };

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM auth_passkeys WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(database)?;
    if count >= MAX_PASSKEYS {
        tx.commit().await.map_err(database)?;
        return Err(AuthError::PasskeyLimit);
    }
    let inserted = sqlx::query(
        "INSERT INTO auth_passkeys (id, user_id, credential_id, credential, name, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (credential_id) DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(passkey.cred_id().as_slice())
    .bind(DbJson(&passkey))
    .bind(name)
    .bind(state.clock.now())
    .execute(&mut *tx)
    .await
    .map_err(database)?
    .rows_affected();
    tx.commit().await.map_err(database)?;
    if inserted == 0 {
        return Err(AuthError::PasskeyFailed);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn verified_registration(
    webauthn: &Webauthn,
    credential: &RegisterPublicKeyCredential,
    flow: &PasskeyRegistration,
) -> Option<Passkey> {
    if credential
        .extensions
        .cred_props
        .as_ref()
        .and_then(|props| props.rk)
        == Some(false)
    {
        return None;
    }
    let passkey = webauthn
        .finish_passkey_registration(credential, flow)
        .ok()?;
    let length = passkey.cred_id().as_slice().len();
    (1..=MAX_CREDENTIAL_ID_BYTES)
        .contains(&length)
        .then_some(passkey)
}

pub(crate) async fn login_start(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: security::AuthJson<LoginStart>,
) -> Result<Json<Start<PublicKeyCredentialRequestOptions>>, AuthError> {
    let webauthn = security::webauthn(&state)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted);
    security::limit(&state, &format!("passkey:login-start:{ip}")).await?;
    let input: LoginStart = security::decode_json(body)?;
    let (options, flow) = webauthn
        .start_discoverable_authentication()
        .map_err(|_| AuthError::PasskeyFailed)?;
    let binding = RawToken::generate()?;
    let cookie = state.cookies.passkey_binding_value(binding.expose())?;
    let binding_hash = TokenHash::of(binding.expose());

    let flow_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO auth_passkey_flows (id, kind, state, binding_hash, return_to, expires_at) \
         VALUES ($1, 'login', $2, $3, $4, $5)",
    )
    .bind(flow_id)
    .bind(DbJson(flow))
    .bind(binding_hash.as_bytes().as_slice())
    .bind(sanitize_return_to(input.return_to.as_deref()))
    .bind(state.clock.now() + CHALLENGE_TTL)
    .execute(&state.pool)
    .await
    .map_err(database)?;

    slot.push(cookie);
    Ok(Json(Start {
        flow_id,
        public_key: options.public_key,
    }))
}

pub(crate) async fn login_finish(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: security::AuthJson<Finish<PublicKeyCredential>>,
) -> Result<Json<SignedIn>, AuthError> {
    let webauthn = security::webauthn(&state)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted);
    security::limit(&state, &format!("passkey:login-finish:{ip}")).await?;
    let input: Finish<PublicKeyCredential> = security::decode_json(body)?;
    let binding = read_cookie(&headers, state.cookies.passkey_binding_name())
        .ok_or(AuthError::SecurityChallengeExpired)?;
    let binding_hash = TokenHash::of(binding);

    let (DbJson(flow), return_to): (DbJson<DiscoverableAuthentication>, String) = sqlx::query_as(
        "DELETE FROM auth_passkey_flows \
         WHERE id = $1 AND kind = 'login' AND binding_hash = $2 AND expires_at > $3 RETURNING state, return_to",
    )
    .bind(input.flow_id)
    .bind(binding_hash.as_bytes().as_slice())
    .bind(state.clock.now())
    .fetch_optional(&state.pool)
    .await
    .map_err(database)?
    .ok_or(AuthError::SecurityChallengeExpired)?;

    let (user_id, credential_id) = webauthn
        .identify_discoverable_authentication(&input.credential)
        .map_err(|_| AuthError::PasskeyFailed)?;
    let user = UserId::from_uuid(user_id);
    let mut tx = state.pool.begin().await.map_err(database)?;
    security::lock_user(&mut tx, user)
        .await
        .map_err(|error| match error {
            AuthError::Unauthenticated => AuthError::PasskeyFailed,
            other => other,
        })?;
    let DbJson(mut passkey): DbJson<Passkey> = sqlx::query_scalar(
        "SELECT credential FROM auth_passkeys WHERE user_id = $1 AND credential_id = $2 FOR UPDATE",
    )
    .bind(user_id)
    .bind(credential_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(database)?
    .ok_or(AuthError::PasskeyFailed)?;

    let result = webauthn
        .finish_discoverable_authentication(
            &input.credential,
            flow,
            &[DiscoverableKey::from(&passkey)],
        )
        .map_err(|_| AuthError::PasskeyFailed)?;
    if passkey
        .update_credential(&result)
        .ok_or(AuthError::PasskeyFailed)?
    {
        sqlx::query(
            "UPDATE auth_passkeys SET credential = $3 WHERE user_id = $1 AND credential_id = $2",
        )
        .bind(user_id)
        .bind(credential_id)
        .bind(DbJson(passkey))
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    }
    let two_factor_required =
        security::complete_primary_locked(&state, &slot, tx, user, AuthMethod::Passkey, &return_to)
            .await?;
    slot.push(state.cookies.clear_passkey_binding()?);
    Ok(Json(SignedIn {
        return_to,
        two_factor_required,
    }))
}

pub(crate) async fn remove(
    State(state): State<AuthState>,
    actor: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AuthError> {
    let mut tx = state.pool.begin().await.map_err(database)?;
    security::lock_user(&mut tx, actor.user_id).await?;
    security::require_recent_in(&mut tx, &state, &actor).await?;
    let id = Uuid::parse_str(&id).map_err(|_| AuthError::InvalidJson)?;
    let user_id = actor.user_id.as_uuid();
    let deleted = sqlx::query("DELETE FROM auth_passkeys WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(database)?
        .rows_affected();
    if deleted != 0 {
        let providers: Vec<&str> = state
            .oauth
            .iter()
            .flat_map(|oauth| oauth.providers.keys().map(|id| id.as_str()))
            .collect();
        let can_sign_in: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM identities WHERE user_id = $1 AND provider = ANY($2))
                 OR EXISTS (SELECT 1 FROM auth_passkeys WHERE user_id = $1)",
        )
        .bind(user_id)
        .bind(&providers)
        .fetch_one(&mut *tx)
        .await
        .map_err(database)?;
        if !can_sign_in {
            return Err(AuthError::LastSignInMethod);
        }
        // A removed credential cannot finish a previously started second-factor check.
        sqlx::query("DELETE FROM auth_step_up WHERE user_id = $1 AND auth_method = 'passkey'")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(database)?;
    }
    tx.commit().await.map_err(database)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::start_registration;
    use uuid::Uuid;
    use webauthn_rs::prelude::{Url, WebauthnBuilder};

    #[test]
    fn registration_requests_a_discoverable_user_verified_passkey() {
        let origin = Url::parse("https://counterspell.example").expect("valid test origin");
        let webauthn = WebauthnBuilder::new("counterspell.example", &origin)
            .expect("valid relying party")
            .build()
            .expect("WebAuthn configuration");
        let (options, _) = start_registration(&webauthn, Uuid::new_v4(), "tester", "Tester", None)
            .expect("registration options");
        let options = serde_json::to_value(options).expect("serializable options");
        let selection = &options["publicKey"]["authenticatorSelection"];
        assert_eq!(selection["residentKey"], "required");
        assert_eq!(selection["requireResidentKey"], true);
        assert_eq!(selection["userVerification"], "required");
    }
}
