//! Database tests for the auth tables (auth T07, requirements R18.1 to R18.5).
//!
//! Each refusal is checked against a valid row that must succeed first, so a missing table cannot
//! make a refusal look like a pass.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_schema -- --ignored pg_`.

use sqlx::PgPool;

/// Inserts a verified user and returns its id.
async fn verified_user(pool: &PgPool, handle: &str) -> sqlx::Result<uuid::Uuid> {
    let (id,): (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(handle)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Inserts a session for `user` that is valid for 30 days.
async fn session_for(pool: &PgPool, user: uuid::Uuid, id: &str) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at,
                               expires_at, absolute_expires_at)
         VALUES ($1, $2, 'magic_link', now(), now(), now() + interval '30 days',
                 now() + interval '90 days')",
    )
    .bind(id)
    .bind(user)
    .execute(pool)
    .await?;
    Ok(())
}

fn session_id(fill: char) -> String {
    std::iter::repeat_n(fill, 64).collect()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_unverified_email_is_refused(pool: PgPool) {
    verified_user(&pool, "thandi").await.expect("valid user first");
    let refused = sqlx::query(
        "INSERT INTO users (email, handle, display_name) VALUES ('x@example.co', 'unverified', 'U')",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err(), "an email without a verification time must be refused");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_password_without_an_email_is_refused(pool: PgPool) {
    verified_user(&pool, "sipho").await.expect("valid user first");
    let refused = sqlx::query(
        "INSERT INTO users (password_hash, handle, display_name) VALUES ('$argon2id$x', 'nopassmail', 'N')",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_second_demo_user_is_refused(pool: PgPool) {
    sqlx::query(
        "INSERT INTO users (email, email_verified_at, handle, display_name, is_demo)
         VALUES ('judge@example.co', now(), 'demo', 'Judge', true)",
    )
    .execute(&pool)
    .await
    .expect("the first demo user is allowed");
    let refused = sqlx::query(
        "INSERT INTO users (email, email_verified_at, handle, display_name, is_demo)
         VALUES ('judge2@example.co', now(), 'demo2', 'Judge 2', true)",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_email_token_longer_than_fifteen_minutes_is_refused(pool: PgPool) {
    sqlx::query(
        "INSERT INTO email_tokens (token_hash, purpose, email, created_at, expires_at)
         VALUES (decode(repeat('01', 32), 'hex'), 'magic_link', 'a@example.co',
                 now(), now() + interval '15 minutes')",
    )
    .execute(&pool)
    .await
    .expect("a 15-minute token is allowed");
    let refused = sqlx::query(
        "INSERT INTO email_tokens (token_hash, purpose, email, created_at, expires_at)
         VALUES (decode(repeat('02', 32), 'hex'), 'magic_link', 'a@example.co',
                 now(), now() + interval '16 minutes')",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_oauth_flow_longer_than_ten_minutes_is_refused(pool: PgPool) {
    sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, binding_hash,
                                  created_at, expires_at)
         VALUES (decode(repeat('01', 32), 'hex'), 'github', 'login', repeat('v', 43),
                 decode(repeat('03', 32), 'hex'), now(), now() + interval '10 minutes')",
    )
    .execute(&pool)
    .await
    .expect("a 10-minute flow is allowed");
    let refused = sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, binding_hash,
                                  created_at, expires_at)
         VALUES (decode(repeat('02', 32), 'hex'), 'github', 'login', repeat('v', 43),
                 decode(repeat('04', 32), 'hex'), now(), now() + interval '11 minutes')",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_login_flow_must_have_a_binding_and_no_session(pool: PgPool) {
    let user = verified_user(&pool, "lindiwe").await.expect("user");
    session_for(&pool, user, &session_id('a')).await.expect("session");
    let link_ok = sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, session_id, user_id,
                                  created_at, expires_at)
         VALUES (decode(repeat('10', 32), 'hex'), 'github', 'link', repeat('v', 43), $1, $2,
                 now(), now() + interval '5 minutes')",
    )
    .bind(session_id('a'))
    .bind(user)
    .execute(&pool)
    .await;
    assert!(link_ok.is_ok(), "a link flow bound to a session is allowed");

    let login_with_session = sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, binding_hash, session_id,
                                  created_at, expires_at)
         VALUES (decode(repeat('11', 32), 'hex'), 'github', 'login', repeat('v', 43),
                 decode(repeat('12', 32), 'hex'), $1, now(), now() + interval '5 minutes')",
    )
    .bind(session_id('a'))
    .execute(&pool)
    .await;
    assert!(login_with_session.is_err());

    let link_without_session = sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier,
                                  created_at, expires_at)
         VALUES (decode(repeat('13', 32), 'hex'), 'github', 'link', repeat('v', 43),
                 now(), now() + interval '5 minutes')",
    )
    .execute(&pool)
    .await;
    assert!(link_without_session.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_session_owner_and_creation_time_cannot_change_and_expiry_cannot_shrink(pool: PgPool) {
    let user = verified_user(&pool, "naledi").await.expect("user");
    let other = verified_user(&pool, "kwame").await.expect("other");
    session_for(&pool, user, &session_id('b')).await.expect("session");

    let owner_change = sqlx::query("UPDATE sessions SET user_id = $1 WHERE id = $2")
        .bind(other)
        .bind(session_id('b'))
        .execute(&pool)
        .await;
    assert!(owner_change.is_err());

    let shrink = sqlx::query("UPDATE sessions SET expires_at = now() WHERE id = $1")
        .bind(session_id('b'))
        .execute(&pool)
        .await;
    assert!(shrink.is_err());

    let renew = sqlx::query(
        "UPDATE sessions SET last_renewed_at = now(), expires_at = now() + interval '30 days'
         WHERE id = $1",
    )
    .bind(session_id('b'))
    .execute(&pool)
    .await;
    assert!(renew.is_ok(), "renewing the expiry forwards is allowed");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_absolute_limit_beyond_ninety_days_is_refused(pool: PgPool) {
    let user = verified_user(&pool, "ama").await.expect("user");
    let refused = sqlx::query(
        "INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at,
                               expires_at, absolute_expires_at)
         VALUES ($1, $2, 'password', now(), now(), now() + interval '30 days',
                 now() + interval '91 days')",
    )
    .bind(session_id('c'))
    .bind(user)
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_deleting_a_user_cascades_to_all_auth_rows(pool: PgPool) {
    let user = verified_user(&pool, "bongani").await.expect("user");
    sqlx::query("INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', '42')")
        .bind(user)
        .execute(&pool)
        .await
        .expect("identity");
    session_for(&pool, user, &session_id('d')).await.expect("session");
    sqlx::query(
        "INSERT INTO email_tokens (token_hash, purpose, email, user_id, created_at, expires_at)
         VALUES (decode(repeat('20', 32), 'hex'), 'link_email', 'x@example.co', $1, now(), now() + interval '5 minutes')",
    )
    .bind(user)
    .execute(&pool)
    .await
    .expect("token");
    sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, session_id, user_id,
                                  created_at, expires_at)
         VALUES (decode(repeat('21', 32), 'hex'), 'google', 'link', repeat('v', 43), $1, $2,
                 now(), now() + interval '5 minutes')",
    )
    .bind(session_id('d'))
    .bind(user)
    .execute(&pool)
    .await
    .expect("flow");

    sqlx::query("DELETE FROM users WHERE id = $1").bind(user).execute(&pool).await.expect("delete");

    for table in ["identities", "sessions", "email_tokens", "oauth_flows"] {
        let (count,): (i64,) = sqlx::query_as(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count, 0, "{table} must be empty after the user is deleted");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_deleting_a_session_cascades_to_its_link_flows(pool: PgPool) {
    let user = verified_user(&pool, "zanele").await.expect("user");
    session_for(&pool, user, &session_id('e')).await.expect("session");
    sqlx::query(
        "INSERT INTO oauth_flows (state_hash, provider, purpose, pkce_verifier, session_id, user_id,
                                  created_at, expires_at)
         VALUES (decode(repeat('30', 32), 'hex'), 'github', 'link', repeat('v', 43), $1, $2,
                 now(), now() + interval '5 minutes')",
    )
    .bind(session_id('e'))
    .bind(user)
    .execute(&pool)
    .await
    .expect("flow");
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id('e'))
        .execute(&pool)
        .await
        .expect("delete session");
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM oauth_flows")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_two_identities_of_one_provider_for_one_user_are_refused(pool: PgPool) {
    let user = verified_user(&pool, "amahle").await.expect("user");
    sqlx::query("INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', '1')")
        .bind(user)
        .execute(&pool)
        .await
        .expect("first identity");
    let refused = sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', '2')",
    )
    .bind(user)
    .execute(&pool)
    .await;
    assert!(refused.is_err());
}
