//! Privilege-matrix tests for the catalogue (problem-catalogue C04, CAT-R1 to CAT-R6).
//!
//! Every table, every privilege and every role is checked against the matrix in
//! `design.md`. The test passes only when the database answers exactly as the matrix says.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_privileges -- --ignored`.

mod catalogue_support;

use sqlx::PgPool;

const SERVER: &str = "counterspell_server";
const JUDGE: &str = "counterspell_judge";

/// Every catalogue table this spec creates.
const ALL_TABLES: [&str; 14] = [
    "tracks",
    "concepts",
    "problems",
    "problem_concepts",
    "problem_versions",
    "problem_examples",
    "problem_starters",
    "problem_hints",
    "hidden_tests",
    "hidden_generation",
    "hidden_sources",
    "reference_solutions",
    "known_ghosts",
    "catalogue_events",
];

/// Tables the server may read (matrix: server SELECT on public catalogue tables).
const SERVER_READS: [&str; 8] = [
    "tracks",
    "concepts",
    "problems",
    "problem_concepts",
    "problem_versions",
    "problem_examples",
    "problem_starters",
    "problem_hints",
];

/// Tables the judge may read (matrix: judge SELECT on the run inputs and hidden data).
// test-change: GHO-6. The Bug Autopsy copies the problem's concept tags into a found ghost, so the
// judge reads `problem_concepts` (additive grant in migration 20261119090000). The old matrix
// predates that; every other row is unchanged.
const JUDGE_READS: [&str; 10] = [
    "problems",
    "problem_concepts",
    "problem_versions",
    "problem_examples",
    "problem_starters",
    "hidden_tests",
    "hidden_generation",
    "hidden_sources",
    "reference_solutions",
    "known_ghosts",
];

const PRIVILEGES: [&str; 4] = ["SELECT", "INSERT", "UPDATE", "DELETE"];

/// Returns true when `role` may use `privilege` on `table`, as the design matrix says.
fn expected(role: &str, table: &str, privilege: &str) -> bool {
    if privilege != "SELECT" {
        return false;
    }
    match role {
        SERVER => SERVER_READS.contains(&table),
        JUDGE => JUDGE_READS.contains(&table),
        _ => false,
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_privileges_match_matrix_for_each_role(pool: PgPool) {
    for table in ALL_TABLES {
        for role in [SERVER, JUDGE] {
            for privilege in PRIVILEGES {
                let actual: bool = sqlx::query_scalar("SELECT has_table_privilege($1, $2, $3)")
                    .bind(role)
                    .bind(table)
                    .bind(privilege)
                    .fetch_one(&pool)
                    .await
                    .expect("has_table_privilege should answer");
                assert_eq!(
                    actual,
                    expected(role, table, privilege),
                    "{role} {privilege} on {table} must match the matrix"
                );
            }
        }
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_privileges_grant_public_nothing(pool: PgPool) {
    for table in ALL_TABLES {
        // aclexplode lists each grant; grantee 0 is the PUBLIC pseudo-role.
        let public_grants: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_class c \
             CROSS JOIN LATERAL aclexplode(coalesce(c.relacl, acldefault('r', c.relowner))) a \
             WHERE c.relname = $1 AND a.grantee = 0",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("the ACL should be readable");
        assert_eq!(public_grants, 0, "PUBLIC must hold no privilege on {table}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_privileges_create_no_new_role(pool: PgPool) {
    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT rolname::text FROM pg_roles WHERE rolname LIKE 'counterspell\\_%' ORDER BY rolname",
    )
    .fetch_all(&pool)
    .await
    .expect("roles should be readable");
    // The cluster's own login user ("counterspell") is not a catalogue role, so only the
    // `counterspell_` prefix is checked.
    assert_eq!(
        roles,
        vec![JUDGE.to_string(), SERVER.to_string()],
        "only the two run-lifecycle roles may exist (CAT-R1)"
    );
}
