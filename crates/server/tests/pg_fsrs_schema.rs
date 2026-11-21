//! PostgreSQL invariants for dashboard-fsrs FSRS-1.
//!
//! Run with `cargo test -p counterspell-server --test pg_fsrs_schema -- --ignored pg_`.

use sqlx::PgPool;
use uuid::Uuid;

const UNIQUE_VIOLATION: &str = "23505";
const CHECK_VIOLATION: &str = "23514";
const FOREIGN_KEY_VIOLATION: &str = "23503";

fn database_code(error: &sqlx::Error) -> String {
    error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .map(std::borrow::Cow::into_owned)
        .expect("rejected statement should return a SQLSTATE")
}

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

async fn seed_user(pool: &PgPool) -> Uuid {
    let user = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'FSRS test user')")
        .bind(user)
        .bind(format!("fsrs_{}", &user.simple().to_string()[..8]))
        .execute(pool)
        .await
        .expect("test user should insert");
    user
}

async fn seed_owner_and_concept(pool: &PgPool) -> (Uuid, &'static str) {
    let user = seed_user(pool).await;
    let concept = "fsrs-test-concept";
    sqlx::query(
        "INSERT INTO concepts (id, name, sort_order) VALUES ($1, 'FSRS test concept', 0)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(concept)
    .execute(pool)
    .await
    .expect("test concept should insert");

    (user, concept)
}

async fn insert_card(pool: &PgPool, user: Uuid, concept: &str) {
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, difficulty, reps, lapses, state, due, version)
         VALUES ($1, $2, 1, 1, 0, 0, 'new', now(), 0)",
    )
    .bind(user)
    .bind(concept)
    .execute(pool)
    .await
    .expect("valid card should insert");
}

async fn insert_history(pool: &PgPool, user: Uuid, concept: &str, grade: &str, interval: &str) {
    sqlx::query(
        "INSERT INTO concept_review_history
         (user_id, concept_id, grade, reviewed_at, previous_card_state, next_card_state, next_interval)
         VALUES ($1, $2, $3, '2026-11-21T12:00:00Z',
                 '{\"state\":\"new\",\"version\":0}'::jsonb,
                 '{\"state\":\"review\",\"version\":1}'::jsonb,
                 $4::interval)",
    )
    .bind(user)
    .bind(concept)
    .bind(grade)
    .bind(interval)
    .execute(pool)
    .await
    .expect("valid review history should insert");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_card_rejects_duplicate_owner_concept(pool: PgPool) {
    let (user, concept) = seed_owner_and_concept(&pool).await;
    let insert = "INSERT INTO concept_cards \
        (user_id, concept_id, stability, difficulty, state, due) \
        VALUES ($1, $2, 0, 0, 'new', now())";

    sqlx::query(insert)
        .bind(user)
        .bind(concept)
        .execute(&pool)
        .await
        .expect("first card should insert");

    let error = sqlx::query(insert)
        .bind(user)
        .bind(concept)
        .execute(&pool)
        .await
        .expect_err("a user may have only one card per concept");
    assert_eq!(database_code(&error), UNIQUE_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_card_rejects_negative_scheduler_fields(pool: PgPool) {
    let (user, concept) = seed_owner_and_concept(&pool).await;
    sqlx::query(
        "INSERT INTO concept_cards \
         (user_id, concept_id, stability, difficulty, reps, lapses, state, due, version) \
         VALUES ($1, $2, 1, 1, 0, 0, 'new', now(), 0)",
    )
    .bind(user)
    .bind(concept)
    .execute(&pool)
    .await
    .expect("valid card should insert");

    for field in ["stability", "difficulty", "reps", "lapses", "version"] {
        let statement =
            format!("UPDATE concept_cards SET {field} = -1 WHERE user_id = $1 AND concept_id = $2");
        let error = sqlx::query(&statement)
            .bind(user)
            .bind(concept)
            .execute(&pool)
            .await
            .expect_err("negative scheduler fields must be rejected");
        assert_eq!(database_code(&error), CHECK_VIOLATION, "field: {field}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_card_requires_valid_owner_and_concept(pool: PgPool) {
    let (user, concept) = seed_owner_and_concept(&pool).await;
    let insert = "INSERT INTO concept_cards \
        (user_id, concept_id, stability, difficulty, state, due) \
        VALUES ($1, $2, 0, 0, 'new', now())";

    let missing_owner = sqlx::query(insert)
        .bind(Uuid::new_v4())
        .bind(concept)
        .execute(&pool)
        .await
        .expect_err("a card without a valid owner must be rejected");
    assert_eq!(database_code(&missing_owner), FOREIGN_KEY_VIOLATION);

    let missing_concept = sqlx::query(insert)
        .bind(user)
        .bind("missing-concept")
        .execute(&pool)
        .await
        .expect_err("a card without a valid concept must be rejected");
    assert_eq!(database_code(&missing_concept), FOREIGN_KEY_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_migrations_apply_twice(pool: PgPool) {
    for file in [
        "20261203120000_concept_cards.sql",
        "20261203121000_concept_review_history.sql",
    ] {
        let path = format!("{}/../../migrations/{file}", env!("CARGO_MANIFEST_DIR"));
        let migration =
            std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path}: {error}"));
        sqlx::raw_sql(&migration)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| panic!("second {file} run: {error}"));
        sqlx::raw_sql(&migration)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| panic!("third {file} run: {error}"));
    }

    let (cards, history): (i64, i64) = sqlx::query_as(
        "SELECT count(*)::bigint, (SELECT count(*)::bigint FROM concept_review_history)
         FROM concept_cards",
    )
    .fetch_one(&pool)
    .await
    .expect("FSRS tables should remain queryable");
    assert_eq!(cards, 0);
    assert_eq!(history, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_server_role_can_use_storage_without_delete(pool: PgPool) {
    for privilege in ["SELECT", "INSERT", "UPDATE"] {
        assert!(
            role_can(&pool, "counterspell_server", "concept_cards", privilege).await,
            "server needs {privilege} on concept_cards"
        );
    }
    for privilege in ["SELECT", "INSERT"] {
        assert!(
            role_can(
                &pool,
                "counterspell_server",
                "concept_review_history",
                privilege
            )
            .await,
            "server needs {privilege} on concept_review_history"
        );
    }
    for table in ["concept_cards", "concept_review_history"] {
        for privilege in ["DELETE", "TRUNCATE"] {
            assert!(
                !role_can(&pool, "counterspell_server", table, privilege).await,
                "server must not have {privilege} on {table}"
            );
        }
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_history_records_review_transition(pool: PgPool) {
    let (user, concept) = seed_owner_and_concept(&pool).await;
    insert_card(&pool, user, concept).await;
    insert_history(&pool, user, concept, "good", "1 day").await;

    let (grade, previous_state, next_state, interval): (String, String, String, String) =
        sqlx::query_as(
            "SELECT grade, previous_card_state->>'state', next_card_state->>'state',
                    next_interval::text
             FROM concept_review_history WHERE user_id = $1 AND concept_id = $2",
        )
        .bind(user)
        .bind(concept)
        .fetch_one(&pool)
        .await
        .expect("review history should read back");

    assert_eq!(grade, "good");
    assert_eq!(previous_state, "new");
    assert_eq!(next_state, "review");
    assert_eq!(interval, "1 day");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_history_requires_card_and_rejects_bad_review_data(pool: PgPool) {
    let (user, concept) = seed_owner_and_concept(&pool).await;
    let without_card = sqlx::query(
        "INSERT INTO concept_review_history
         (user_id, concept_id, grade, reviewed_at, previous_card_state, next_card_state, next_interval)
         VALUES ($1, $2, 'good', now(), '{}'::jsonb, '{}'::jsonb, interval '1 day')",
    )
    .bind(user)
    .bind(concept)
    .execute(&pool)
    .await
    .expect_err("history must belong to an existing card");
    assert_eq!(database_code(&without_card), FOREIGN_KEY_VIOLATION);

    insert_card(&pool, user, concept).await;
    let bad_grade = sqlx::query(
        "INSERT INTO concept_review_history
         (user_id, concept_id, grade, reviewed_at, previous_card_state, next_card_state, next_interval)
         VALUES ($1, $2, 'again-ish', now(), '{}'::jsonb, '{}'::jsonb, interval '1 day')",
    )
    .bind(user)
    .bind(concept)
    .execute(&pool)
    .await
    .expect_err("unknown FSRS grades must be rejected");
    assert_eq!(database_code(&bad_grade), CHECK_VIOLATION);

    let negative_interval = sqlx::query(
        "INSERT INTO concept_review_history
         (user_id, concept_id, grade, reviewed_at, previous_card_state, next_card_state, next_interval)
         VALUES ($1, $2, 'good', now(), '{}'::jsonb, '{}'::jsonb, interval '-1 second')",
    )
    .bind(user)
    .bind(concept)
    .execute(&pool)
    .await
    .expect_err("negative review intervals must be rejected");
    assert_eq!(database_code(&negative_interval), CHECK_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_fsrs_user_delete_cascades_only_owned_cards_and_history(pool: PgPool) {
    let (alice, concept) = seed_owner_and_concept(&pool).await;
    let (bob, same_concept) = seed_owner_and_concept(&pool).await;
    assert_eq!(concept, same_concept);

    insert_card(&pool, alice, concept).await;
    insert_history(&pool, alice, concept, "again", "0 seconds").await;
    insert_card(&pool, bob, concept).await;
    insert_history(&pool, bob, concept, "easy", "2 days").await;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(alice)
        .execute(&pool)
        .await
        .expect("deleting the owner should cascade their FSRS records");

    for table in ["concept_cards", "concept_review_history"] {
        let statement = format!("SELECT count(*) FROM {table} WHERE user_id = $1");
        let (alice_rows,): (i64,) = sqlx::query_as(&statement)
            .bind(alice)
            .fetch_one(&pool)
            .await
            .expect("deleted owner's row count");
        let (bob_rows,): (i64,) = sqlx::query_as(&statement)
            .bind(bob)
            .fetch_one(&pool)
            .await
            .expect("other owner's row count");
        assert_eq!(alice_rows, 0, "{table} should cascade for the deleted user");
        assert_eq!(bob_rows, 1, "{table} must retain the other user's row");
    }
}
