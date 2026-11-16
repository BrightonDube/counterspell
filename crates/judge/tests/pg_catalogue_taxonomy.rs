//! Taxonomy sync (problem-catalogue C20, CAT-T3, CAT-T2 database check).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_taxonomy -- --ignored`.

use counterspell_judge::catalogue::taxonomy::{parse_taxonomy, sync_taxonomy, Taxonomy};
use sqlx::PgPool;

fn taxonomy(text: &str) -> Taxonomy {
    parse_taxonomy(text).expect("fixture taxonomy parses")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_taxonomy(pool: PgPool) {
    let first = taxonomy(
        "[[tracks]]\nslug = \"arrays\"\nname = \"Arrays\"\n\
         [[concepts]]\nslug = \"hash-maps\"\nname = \"Hash maps\"\n\
         [[concepts]]\nslug = \"two-pointers\"\nname = \"Two pointers\"\n",
    );
    sync_taxonomy(&pool, &first).await.expect("first sync");

    // A rename updates the display name of the same slug.
    let renamed = taxonomy(
        "[[tracks]]\nslug = \"arrays\"\nname = \"Arrays and lists\"\n\
         [[concepts]]\nslug = \"hash-maps\"\nname = \"Hash tables\"\n",
    );
    sync_taxonomy(&pool, &renamed).await.expect("second sync");

    let hash_name: String = sqlx::query_scalar("SELECT name FROM concepts WHERE id = 'hash-maps'")
        .fetch_one(&pool)
        .await
        .expect("renamed concept row");
    assert_eq!(hash_name, "Hash tables");

    // A removed entry is kept, never deleted.
    let kept: i64 = sqlx::query_scalar("SELECT count(*) FROM concepts WHERE id = 'two-pointers'")
        .fetch_one(&pool)
        .await
        .expect("count removed concept");
    assert_eq!(kept, 1, "removed concept must stay in the table");

    // Running the same sync again creates no duplicates.
    sync_taxonomy(&pool, &renamed).await.expect("third sync");
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM concepts")
        .fetch_one(&pool)
        .await
        .expect("count concepts");
    assert_eq!(total, 2);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_taxonomy_unknown_concept_refused(pool: PgPool) {
    sync_taxonomy(
        &pool,
        &taxonomy("[[tracks]]\nslug = \"arrays\"\nname = \"Arrays\"\n"),
    )
    .await
    .expect("track sync");

    // A problem tagged with a concept that the taxonomy does not define is refused by the key.
    sqlx::query("INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ('count-pairs', 'Count pairs', 'arrays', 'easy')")
        .execute(&pool)
        .await
        .expect("problem row");
    let refused = sqlx::query(
        "INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ('count-pairs', 'no-such-concept')",
    )
    .execute(&pool)
    .await;
    assert!(refused.is_err(), "unknown concept must be refused");
}
