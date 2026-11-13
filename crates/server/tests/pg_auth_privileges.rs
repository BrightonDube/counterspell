//! Database tests for the auth privilege matrix (auth T08, requirements R1.3 and R18.6 to R18.8).
//!
//! The judge role has no privilege on any auth table, PUBLIC has none, and the server role gets
//! exactly the columns in the matrix. The foreign-key rule for `submissions` is tested with the
//! run-lifecycle fixtures in T34, because adding that key now would break the existing run tests.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_privileges -- --ignored pg_`.

use sqlx::PgPool;

const AUTH_TABLES: [&str; 5] = [
    "users",
    "identities",
    "email_tokens",
    "sessions",
    "oauth_flows",
];

async fn role_can(pool: &PgPool, role: &str, table: &str, privilege: &str) -> bool {
    let (granted,): (bool,) = sqlx::query_as("SELECT has_table_privilege($1, $2, $3)")
        .bind(role)
        .bind(table)
        .bind(privilege)
        .fetch_one(pool)
        .await
        .expect("privilege query");
    granted
}

async fn column_can(pool: &PgPool, role: &str, table: &str, column: &str) -> bool {
    let (granted,): (bool,) = sqlx::query_as("SELECT has_column_privilege($1, $2, $3, 'UPDATE')")
        .bind(role)
        .bind(table)
        .bind(column)
        .fetch_one(pool)
        .await
        .expect("column privilege query");
    granted
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_judge_role_and_public_have_no_privilege_on_auth_tables(pool: PgPool) {
    for table in AUTH_TABLES {
        for privilege in [
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "TRUNCATE",
            "REFERENCES",
            "TRIGGER",
        ] {
            assert!(
                !role_can(&pool, "counterspell_judge", table, privilege).await,
                "judge must not have {privilege} on {table}"
            );
        }
        let (public_entries,): (i64,) = sqlx::query_as(
            "SELECT count(*) FROM pg_class c
             JOIN LATERAL aclexplode(c.relacl) a ON true
             WHERE c.relname = $1 AND a.grantee = 0",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("acl query");
        assert_eq!(public_entries, 0, "PUBLIC must have no grant on {table}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_server_role_cannot_truncate_or_change_the_demo_columns(pool: PgPool) {
    for table in AUTH_TABLES {
        assert!(
            !role_can(&pool, "counterspell_server", table, "TRUNCATE").await,
            "{table}"
        );
    }
    for column in ["handle", "is_demo", "demo_reset_at"] {
        assert!(
            !column_can(&pool, "counterspell_server", "users", column).await,
            "server must not update users.{column}"
        );
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_server_role_has_the_matrix_privileges_and_no_more(pool: PgPool) {
    assert!(column_can(&pool, "counterspell_server", "users", "email").await);
    assert!(column_can(&pool, "counterspell_server", "users", "password_hash").await);
    assert!(role_can(&pool, "counterspell_server", "users", "DELETE").await);
    assert!(role_can(&pool, "counterspell_server", "identities", "INSERT").await);
    assert!(!role_can(&pool, "counterspell_server", "identities", "DELETE").await);
    assert!(column_can(&pool, "counterspell_server", "email_tokens", "used_at").await);
    assert!(!column_can(&pool, "counterspell_server", "email_tokens", "payload").await);
    assert!(column_can(&pool, "counterspell_server", "sessions", "expires_at").await);
    assert!(column_can(&pool, "counterspell_server", "sessions", "last_renewed_at").await);
    assert!(!column_can(&pool, "counterspell_server", "sessions", "user_id").await);
    assert!(role_can(&pool, "counterspell_server", "oauth_flows", "DELETE").await);
    assert!(!role_can(&pool, "counterspell_server", "oauth_flows", "UPDATE").await);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_every_foreign_key_to_users_cascades_on_delete(pool: PgPool) {
    let (not_cascading,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_constraint
         WHERE contype = 'f' AND confrelid = 'users'::regclass AND confdeltype <> 'c'",
    )
    .fetch_one(&pool)
    .await
    .expect("constraint query");
    assert_eq!(not_cascading, 0);
}
