//! OAuth flow store over real Postgres (auth T18, property P7, requirements R6.1, R6.3, R8.1,
//! R8.2 and R8.7).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_flow -- --include-ignored`.

use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    clock::ManualClock,
    oauth::{flow::FlowStore, ProviderId},
    session::{AuthMethod, SessionService},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

fn clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ))
}

async fn user(pool: &PgPool, handle: &str) -> UserId {
    let (id,): (sqlx::types::Uuid,) =
        sqlx::query_as("INSERT INTO users (handle, display_name) VALUES ($1, $1) RETURNING id")
            .bind(handle)
            .fetch_one(pool)
            .await
            .expect("insert a user");
    UserId::from_uuid(id)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_the_stored_verifier_hashes_to_the_challenge(pool: PgPool) {
    let store = FlowStore::new(pool, clock());

    let created = store
        .create_login(ProviderId::GitHub, "/problems")
        .await
        .expect("create a login flow");
    let flow = store
        .consume(created.state.expose(), ProviderId::GitHub)
        .await
        .expect("consume")
        .expect("the flow exists");

    let hashed = URL_SAFE_NO_PAD.encode(Sha256::digest(flow.pkce_verifier().as_bytes()));
    assert_eq!(hashed, created.challenge);
    assert_eq!(created.challenge.len(), 43);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_works_exactly_once(pool: PgPool) {
    let store = FlowStore::new(pool, clock());
    let created = store
        .create_login(ProviderId::Google, "/")
        .await
        .expect("create");

    let first = store
        .consume(created.state.expose(), ProviderId::Google)
        .await
        .expect("consume");
    let second = store
        .consume(created.state.expose(), ProviderId::Google)
        .await
        .expect("consume again");

    assert!(first.is_some());
    assert!(second.is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_a_wrong_provider_fails_and_still_uses_the_flow(pool: PgPool) {
    let store = FlowStore::new(pool, clock());
    let created = store
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("create");

    let wrong = store
        .consume(created.state.expose(), ProviderId::Google)
        .await
        .expect("consume");
    let right = store
        .consume(created.state.expose(), ProviderId::GitHub)
        .await
        .expect("consume again");

    assert!(wrong.is_none());
    assert!(
        right.is_none(),
        "the failed attempt must have used the flow"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_an_altered_state_finds_nothing(pool: PgPool) {
    let store = FlowStore::new(pool, clock());
    let created = store
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("create");
    let altered = format!("{}x", created.state.expose());

    let found = store
        .consume(&altered, ProviderId::GitHub)
        .await
        .expect("consume");

    assert!(found.is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_expires_ten_minutes_after_creation(pool: PgPool) {
    let clock = clock();
    let store = FlowStore::new(pool, clock.clone());
    let created = store
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("create");

    clock.advance(Duration::minutes(10));
    let found = store
        .consume(created.state.expose(), ProviderId::GitHub)
        .await
        .expect("consume");

    assert!(found.is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_a_login_needs_its_own_binding_cookie(pool: PgPool) {
    let store = FlowStore::new(pool, clock());
    let created = store
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("create");
    let flow = store
        .consume(created.state.expose(), ProviderId::GitHub)
        .await
        .expect("consume")
        .expect("the flow exists");

    let binding = created
        .binding
        .as_ref()
        .expect("login flows carry a binding");
    assert!(flow.binding_matches(Some(binding.expose())));
    assert!(!flow.binding_matches(None));
    assert!(!flow.binding_matches(Some("some-other-binding-value-that-is-not-it-0000")));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_flow_a_link_is_bound_to_its_session_and_ends_with_it(pool: PgPool) {
    let clock = clock();
    let sessions = SessionService::new(pool.clone(), clock.clone());
    let owner = user(&pool, "linker").await;
    let session = sessions
        .create(owner, AuthMethod::MagicLink)
        .await
        .expect("create a session");
    let store = FlowStore::new(pool.clone(), clock);

    let created = store
        .create_link(ProviderId::Google, &session.key, owner, "/settings/account")
        .await
        .expect("create a link flow");
    let flow = store
        .consume(created.state.expose(), ProviderId::Google)
        .await
        .expect("consume")
        .expect("the link flow exists");
    assert_eq!(flow.user(), Some(owner));
    assert_eq!(flow.session_key(), Some(session.key.as_str()));

    let second_link = store
        .create_link(ProviderId::Google, &session.key, owner, "/")
        .await
        .expect("create another link flow");
    sessions.revoke(&session.key).await.expect("revoke");
    let after_logout = store
        .consume(second_link.state.expose(), ProviderId::Google)
        .await
        .expect("consume");

    assert!(after_logout.is_none(), "a flow ends with its session");
}
