//! Store tests for the hints tables (hints spec H11). Each `pg_` test runs the real migrations on a
//! fresh database. They need `DATABASE_URL` and are run with `--ignored`.

use counterspell_common::ids::UserId;
use counterspell_server::hints::store;
use counterspell_server::testkit::ensure_user;
use sqlx::PgPool;
use uuid::Uuid;

/// Creates a learner and one finished submission for them, and returns both ids.
async fn seed_finished(pool: &PgPool) -> (Uuid, Uuid) {
    let user = Uuid::new_v4();
    ensure_user(pool, UserId::from_uuid(user))
        .await
        .expect("a user row can be created");
    let submission = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', $3, 'completed', 'wrong_answer', now())",
    )
    .bind(submission)
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(pool)
    .await
    .expect("a finished submission row can be created");
    (user, submission)
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_two_concurrent_reveals_store_one_row_with_one_text(pool: PgPool) {
    let (user, submission) = seed_finished(&pool).await;

    let (first, second) = tokio::join!(
        store::reveal_ladder(&pool, user, submission, 1, "The first text"),
        store::reveal_ladder(&pool, user, submission, 1, "The second text"),
    );

    let first = first.expect("the first reveal succeeds");
    let second = second.expect("the second reveal succeeds");
    assert_eq!(first.body, second.body, "both calls return the stored text");
    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM hints WHERE submission_id = $1 AND source = 'ladder'",
    )
    .bind(submission)
    .fetch_one(&pool)
    .await
    .expect("the count can be read");
    assert_eq!(rows, 1);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_a_repeated_reveal_returns_the_stored_text_and_no_new_row(pool: PgPool) {
    let (user, submission) = seed_finished(&pool).await;
    store::reveal_ladder(&pool, user, submission, 1, "Stored once")
        .await
        .expect("the first reveal succeeds");

    let again = store::reveal_ladder(&pool, user, submission, 1, "A later text")
        .await
        .expect("the repeat succeeds");

    assert_eq!(again.body, "Stored once");
    let revealed = store::revealed_rungs(&pool, submission)
        .await
        .expect("the revealed rungs can be read");
    assert_eq!(revealed.into_iter().collect::<Vec<u8>>(), vec![1]);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_a_submission_is_visible_only_to_its_finished_owner(pool: PgPool) {
    let (owner, submission) = seed_finished(&pool).await;
    let stranger = Uuid::new_v4();
    let unknown = Uuid::new_v4();
    let queued = Uuid::new_v4();
    ensure_user(&pool, UserId::from_uuid(stranger))
        .await
        .expect("a second user row can be created");
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', $3, 'queued')",
    )
    .bind(queued)
    .bind(owner)
    .bind(vec![0u8; 32])
    .execute(&pool)
    .await
    .expect("an unfinished submission row can be created");

    let visible = store::load_submission(&pool, owner, submission)
        .await
        .expect("query runs");
    let other_user = store::load_submission(&pool, stranger, submission)
        .await
        .expect("query runs");
    let no_such = store::load_submission(&pool, owner, unknown)
        .await
        .expect("query runs");
    let unfinished = store::load_submission(&pool, owner, queued)
        .await
        .expect("query runs");

    assert!(visible.is_some(), "the owner sees a finished submission");
    assert!(other_user.is_none(), "another user sees nothing");
    assert!(no_such.is_none(), "an unknown id sees nothing");
    assert!(
        unfinished.is_none(),
        "an unfinished submission is not open for hints"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_an_ai_hint_is_stored_once_and_read_back(pool: PgPool) {
    let (user, submission) = seed_finished(&pool).await;
    let new = store::NewAiHint {
        user,
        submission,
        rung: 2,
        provider: "openai",
        model: "gpt-6-luna",
        body: "Look at the second loop.",
        tokens_in: 120,
        tokens_out: 30,
        cost_micros: 25,
        cost_estimated: false,
        cache_key: [9u8; 32],
    };

    let stored = store::insert_ai(&pool, &new)
        .await
        .expect("the AI hint is stored");
    let duplicate = store::insert_ai(&pool, &new)
        .await
        .expect("a repeat is not an error");
    let found = store::find_ai(&pool, submission, 2)
        .await
        .expect("the lookup runs");

    assert_eq!(stored.body, "Look at the second loop.");
    assert_eq!(duplicate.body, stored.body);
    assert_eq!(
        found.map(|hint| hint.body),
        Some("Look at the second loop.".to_owned())
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_the_ghost_is_none_while_no_ghost_table_exists(pool: PgPool) {
    let (_user, submission) = seed_finished(&pool).await;

    let ghost = store::ghost_for(&pool, submission)
        .await
        .expect("the lookup runs");

    assert!(
        ghost.is_none(),
        "with no ghosts table the prompt has no ghost (R3.5)"
    );
}
