//! Integration tests against a throwaway Postgres, plus the daily maths.

use sqlx::{PgPool, Row};

// The 12-row synthetic fixture ships at crates/server/tests/fixtures.
const CSV: &str = include_str!("fixtures/breakroom-sample.csv");

async fn import_fixture(pool: &PgPool) {
    for line in CSV.lines().skip(1) {
        let c: Vec<&str> = line.split(',').collect();
        sqlx::query(
                "INSERT INTO puzzles (id, fen, moves, rating, themes, popularity,                  nb_plays, source_date) VALUES ($1,$2,$3,$4,$5,$6,$7,'2026-11-01')                  ON CONFLICT (id) DO NOTHING",
            )
            .bind(c[0])
            .bind(c[1])
            .bind(c[2])
            .bind(c[3].parse::<i32>().expect("rating"))
            .bind(c[7])
            .bind(c[5].parse::<i32>().expect("popularity"))
            .bind(c[6].parse::<i32>().expect("plays"))
            .execute(pool)
            .await
            .expect("insert puzzle");
    }
}

// test-change: BRK-1. Run against the isolated Postgres test service.
#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_breakroom_import_and_daily(pool: PgPool) {
    import_fixture(&pool).await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM puzzles")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 12);
    let row = sqlx::query(
        "SELECT id, fen, moves, rating, themes FROM puzzles ORDER BY id OFFSET 0 LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("row");
    assert_eq!(row.get::<String, _>("id"), "SYN01");
}

// test-change: BRK-1. Run against the isolated Postgres test service.
#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_breakroom_one_puzzle_score_per_day(pool: PgPool) {
    let uid: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (handle, display_name) VALUES ('brk_a', 'Brk A')              RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| panic!("user insert"));
    let meta = serde_json::json!({ "date": "2026-11-13", "mistakes": 0 });
    let first: Option<i64> = sqlx::query_scalar(
            "INSERT INTO game_scores (user_id, game, score, meta) VALUES ($1,'chess_puzzle',100,$2)              ON CONFLICT DO NOTHING RETURNING id",
        )
        .bind(uid)
        .bind(&meta)
        .fetch_optional(&pool)
        .await
        .expect("insert");
    assert!(first.is_some());
    let dup: Option<i64> = sqlx::query_scalar(
            "INSERT INTO game_scores (user_id, game, score, meta) VALUES ($1,'chess_puzzle',100,$2)              ON CONFLICT DO NOTHING RETURNING id",
        )
        .bind(uid)
        .bind(&meta)
        .fetch_optional(&pool)
        .await
        .expect("dup insert");
    assert_eq!(dup, None, "partial unique index rejects the same date");
}

// test-change: BRK-1. Run against the isolated Postgres test service.
#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_breakroom_delete_user_cascades_scores(pool: PgPool) {
    let uid: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (handle, display_name) VALUES ('brk_b', 'Brk B')              RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("user insert");
    sqlx::query("INSERT INTO game_scores (user_id, game, score, meta) VALUES ($1,'brain',50,'{}')")
        .bind(uid)
        .execute(&pool)
        .await
        .expect("score insert");
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&pool)
        .await
        .expect("delete user");
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM game_scores")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(left, 0);
}
