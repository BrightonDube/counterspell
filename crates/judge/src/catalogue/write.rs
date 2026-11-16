//! Static import writes (problem-catalogue C14, CAT-I10, CAT-I11, CAT-I12, CAT-P5).
//!
//! `write_static` writes one pack as one transaction. It locks the problem row, plans against the
//! latest stored version, and only then inserts. Any error rolls the whole pack back, so a failed
//! import never leaves a version without its children. Refusals and unchanged packs write nothing.
//!
//! The function takes a [`StaticImport`] that is already built and checked. Reading pack files
//! and running the sandbox are other tasks.

use crate::catalogue::plan::{self, Incoming, Plan, Refusal, StoredVersion};
use crate::catalogue::sandbox_fill::{CaseSource, SandboxFill};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A known ghost: a failing input that the pack documents, with its authored explanation.
#[derive(Debug, Clone, PartialEq)]
pub struct KnownGhost {
    /// Stable id within the pack.
    pub ghost_id: String,
    /// One of `wrong`, `crash` or `timeout`.
    pub kind: String,
    /// The failing input.
    pub input: Value,
    /// What the program returned, when there is a value.
    pub got: Option<Value>,
    /// The exception class for a crash.
    pub exception_class: Option<String>,
    /// One plain-English line, at most 400 bytes.
    pub explanation: String,
    /// Concept ids the ghost teaches.
    pub concepts: Vec<String>,
}

/// Everything one static version needs. Hidden data is not part of this struct.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticImport {
    /// The problem slug.
    pub slug: String,
    /// The version, equal to `pack_version` (D11).
    pub version: i32,
    /// The combined content hash from the manifest.
    pub content_sha256: [u8; 32],
    /// Public files only, as `public/<path>` to hex hash (CAT-PT9).
    pub manifest_public: BTreeMap<String, String>,
    /// Display title.
    pub title: String,
    /// Track slug.
    pub track: String,
    /// `easy`, `medium` or `hard`.
    pub difficulty: String,
    /// The statement in Markdown.
    pub statement_markdown: String,
    /// Bullet items from the Constraints section.
    pub constraints: Vec<String>,
    /// Function name, parameters and return type as JSON.
    pub signature: Value,
    /// The engine input schema as JSON.
    pub input_schema: Value,
    /// `exact` or `unordered`.
    pub comparator: String,
    /// Time and memory limits as JSON.
    pub limits: Value,
    /// Declared complexity such as `n log n`, when present.
    pub reference_complexity: Option<String>,
    /// Bench specification as JSON, when present.
    pub bench: Option<Value>,
    /// A short note on the complexity, when present.
    pub complexity_note: Option<String>,
    /// Engine version recorded for this version.
    pub engine_version: String,
    /// Generator version recorded for this version.
    pub generator_version: String,
    /// Number of hidden tests the sandbox step will produce (5 to 80).
    pub hidden_count: i32,
    /// Concept ids for this problem. The set replaces the stored set.
    pub concepts: Vec<String>,
    /// Sample inputs and expected outputs, in order.
    pub examples: Vec<(Value, Value)>,
    /// Starter code as `(lang, code)`.
    pub starters: Vec<(String, String)>,
    /// Hint rungs as `(rung, body)`.
    pub hints: Vec<(i32, String)>,
    /// The Python reference solution. Judge only.
    pub reference_code: String,
    /// Known ghosts. Judge only.
    pub known_ghosts: Vec<KnownGhost>,
}

/// What a write did. Refusals are values, not errors, so the caller can report them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteOutcome {
    /// A new version was written.
    Created {
        /// The version number written.
        version: i32,
    },
    /// The stored version already matches. Nothing was written.
    Unchanged {
        /// The stored version number.
        version: i32,
    },
    /// The planner refused the import. Nothing was written.
    Refused(Refusal),
}

/// A write that failed. The transaction was rolled back.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// A database statement failed, usually a CHECK or trigger.
    #[error("database write failed")]
    Database(#[from] sqlx::Error),
    /// A stored version could not be read back for planning.
    #[error("stored version {version} is unreadable: {reason}")]
    CorruptStoredVersion {
        /// The stored version number.
        version: i32,
        /// A fixed reason string.
        reason: &'static str,
    },
}

/// Writes one static version of `import` in a single transaction.
///
/// # Errors
///
/// Returns [`WriteError::Database`] when any statement fails, with nothing written. Returns
/// [`WriteError::CorruptStoredVersion`] when the latest stored version cannot be read.
pub async fn write_static(
    pool: &PgPool,
    import: &StaticImport,
    operator: &str,
) -> Result<WriteOutcome, WriteError> {
    let mut tx = pool.begin().await?;
    lock_problem(&mut tx, import).await?;
    let latest = latest_version(&mut tx, &import.slug).await?;
    match plan::plan(latest.as_ref(), &incoming_of(import)) {
        Plan::Unchanged => {
            let version = latest.map_or(import.version, |stored| stored.version);
            tx.rollback().await?;
            Ok(WriteOutcome::Unchanged { version })
        }
        Plan::Refuse(refusal) => {
            tx.rollback().await?;
            Ok(WriteOutcome::Refused(refusal))
        }
        Plan::Create => {
            insert_static(&mut tx, import, operator).await?;
            tx.commit().await?;
            Ok(WriteOutcome::Created {
                version: import.version,
            })
        }
    }
}

/// Builds the planner's view of `import`.
fn incoming_of(import: &StaticImport) -> Incoming {
    Incoming {
        pack_version: import.version,
        hash: import.content_sha256,
        files: import
            .manifest_public
            .iter()
            .map(|(path, hash)| (path.clone(), hash.clone()))
            .collect(),
    }
}

/// Which public files were added, removed or changed between two stored and incoming manifests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestDiff {
    /// Paths present only in the incoming pack.
    pub added: Vec<String>,
    /// Paths present only in the stored version.
    pub removed: Vec<String>,
    /// Paths present in both with different hashes.
    pub changed: Vec<String>,
}

/// What a dry run found. Nothing was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    /// What a real import would do.
    pub plan: Plan,
    /// The manifest difference against the latest stored version.
    pub diff: ManifestDiff,
    /// `(stored, current)` when the engine's generator version changed (CAT-I19).
    pub generator_change: Option<(String, String)>,
}

/// Plans `import` without writing. The transaction is read only, so the database itself refuses
/// any write made by mistake (CAT-I13).
///
/// # Errors
///
/// Returns [`WriteError`] when the stored version cannot be read.
pub async fn dry_run_static(
    pool: &PgPool,
    import: &StaticImport,
) -> Result<DryRunReport, WriteError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await?;
    let latest = latest_version(&mut tx, &import.slug).await?;
    let stored_generator = stored_generator_version(&mut tx, &import.slug).await?;
    tx.rollback().await?;

    let incoming = incoming_of(import);
    let stored_files: &[(String, String)] = latest.as_ref().map_or(&[], |stored| &stored.files);
    Ok(DryRunReport {
        plan: plan::plan(latest.as_ref(), &incoming),
        diff: diff_files(stored_files, &incoming.files),
        generator_change: stored_generator
            .filter(|stored| *stored != import.generator_version)
            .map(|stored| (stored, import.generator_version.clone())),
    })
}

/// Compares two `(path, hash)` lists. Both are sorted by path in the result.
#[must_use]
pub fn diff_files(stored: &[(String, String)], incoming: &[(String, String)]) -> ManifestDiff {
    let stored_map: BTreeMap<&str, &str> = stored
        .iter()
        .map(|(path, hash)| (path.as_str(), hash.as_str()))
        .collect();
    let incoming_map: BTreeMap<&str, &str> = incoming
        .iter()
        .map(|(path, hash)| (path.as_str(), hash.as_str()))
        .collect();
    let mut diff = ManifestDiff::default();
    for (path, hash) in &incoming_map {
        match stored_map.get(path) {
            None => diff.added.push((*path).to_string()),
            Some(old) if old != hash => diff.changed.push((*path).to_string()),
            Some(_) => {}
        }
    }
    for path in stored_map.keys() {
        if !incoming_map.contains_key(path) {
            diff.removed.push((*path).to_string());
        }
    }
    diff
}

/// Everything the one-time sandbox write stores (C17, CAT-I14, CAT-S2, CAT-S3).
///
/// The hidden cases come from [`crate::catalogue::sandbox_fill::sandbox_fill`]; they
/// live only in this struct until the write commits. Nothing else may carry them.
#[derive(Debug, Clone, PartialEq)]
pub struct SandboxImport {
    /// The problem slug.
    pub slug: String,
    /// The version this fill validates.
    pub version: i32,
    /// The produced hidden cases and the image digest that computed them.
    pub fill: SandboxFill,
    /// The generator seed recorded in `hidden_generation`.
    pub generator_seed: i64,
    /// The generator version recorded in `hidden_generation`.
    pub generator_version: String,
    /// Private-tree file hashes as `(private/<path>, sha256)`, for `hidden_sources`.
    pub sources: Vec<(String, [u8; 32])>,
}

/// What a sandbox write did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxOutcome {
    /// The hidden data and the sandbox stamp were written.
    Stamped {
        /// The stamped version number.
        version: i32,
    },
    /// The version was already sandbox-validated; nothing was written (CAT-S2).
    Unchanged {
        /// The stored version number.
        version: i32,
    },
    /// No such `(slug, version)` row exists; nothing was written.
    Missing {
        /// The requested version number.
        version: i32,
    },
}

/// Writes one sandbox fill in a single transaction (CAT-I14).
///
/// The order inside the transaction is fixed: `hidden_tests` (curated rows keep
/// their lead over generated ones), `hidden_generation`, `hidden_sources`, then the
/// sandbox stamp. The version guard refuses the stamp unless hidden tests exist,
/// so the stamp can never name an empty fill. A second call for an already-stamped
/// version returns [`SandboxOutcome::Unchanged`] without touching a row (CAT-S2).
///
/// # Errors
///
/// Returns [`WriteError::Database`] when any statement fails, with nothing written.
pub async fn write_sandbox(
    pool: &PgPool,
    import: &SandboxImport,
    operator: &str,
) -> Result<SandboxOutcome, WriteError> {
    let mut tx = pool.begin().await?;
    // The problem row lock serializes this write against every other catalogue
    // write for the slug, so two sandbox fills cannot race the stamp (CAT-I12).
    let locked: Option<(String,)> =
        sqlx::query_as("SELECT slug FROM problems WHERE slug = $1 FOR UPDATE")
            .bind(&import.slug)
            .fetch_optional(&mut *tx)
            .await?;
    let validated: Option<bool> = sqlx::query_scalar(
        "SELECT sandbox_validated_at IS NOT NULL FROM problem_versions \
         WHERE problem_slug = $1 AND version = $2",
    )
    .bind(&import.slug)
    .bind(import.version)
    .fetch_optional(&mut *tx)
    .await?;
    let outcome = match (locked, validated) {
        (None, _) | (_, None) => SandboxOutcome::Missing {
            version: import.version,
        },
        (_, Some(true)) => SandboxOutcome::Unchanged {
            version: import.version,
        },
        (_, Some(false)) => {
            insert_sandbox(&mut tx, import, operator).await?;
            SandboxOutcome::Stamped {
                version: import.version,
            }
        }
    };
    match outcome {
        SandboxOutcome::Stamped { version } => {
            tx.commit().await?;
            Ok(SandboxOutcome::Stamped { version })
        }
        unchanged_or_missing => {
            tx.rollback().await?;
            Ok(unchanged_or_missing)
        }
    }
}

/// Inserts the hidden rows, stamps the version and appends the audit event.
///
/// `hidden_tests` ordinals continue after the version's samples so one ordinal
/// space covers the whole test list ("samples first, then curated, then
/// generated"). The event carries counts only (CAT-S6).
async fn insert_sandbox(
    conn: &mut PgConnection,
    import: &SandboxImport,
    operator: &str,
) -> Result<(), WriteError> {
    let samples: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM problem_examples WHERE problem_slug = $1 AND version = $2",
    )
    .bind(&import.slug)
    .bind(import.version)
    .fetch_one(&mut *conn)
    .await?;
    let base = usize::try_from(samples).unwrap_or(usize::MAX);
    let mut curated = 0_usize;
    let mut generated = 0_usize;
    for (index, case) in import.fill.hidden.iter().enumerate() {
        let source = match case.source {
            CaseSource::Curated => {
                curated += 1;
                "curated"
            }
            CaseSource::Generated => {
                generated += 1;
                "generated"
            }
            // The fill never produces a sample case; if a caller built one the
            // hidden_tests CHECK refuses the insert and rolls the write back.
            CaseSource::Sample => "sample",
        };
        sqlx::query(
            "INSERT INTO hidden_tests (problem_slug, version, ordinal, source, input, expected) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(ordinal(base.saturating_add(index)))
        .bind(source)
        .bind(&case.input)
        .bind(&case.expected)
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query(
        "INSERT INTO hidden_generation (problem_slug, version, generator_seed, generator_version) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(import.generator_seed)
    .bind(&import.generator_version)
    .execute(&mut *conn)
    .await?;
    for (path, sha256) in &import.sources {
        sqlx::query(
            "INSERT INTO hidden_sources (problem_slug, version, path, sha256) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(path)
        .bind(sha256.as_slice())
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query(
        "UPDATE problem_versions SET sandbox_validated_at = now(), sandbox_image_digest = $3 \
         WHERE problem_slug = $1 AND version = $2",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(&import.fill.digest)
    .execute(&mut *conn)
    .await?;
    let detail = serde_json::json!({
        "hidden_curated": curated,
        "hidden_generated": generated,
        "hidden_sources": import.sources.len(),
    });
    sqlx::query(
        "INSERT INTO catalogue_events (problem_slug, action, version, operator, detail) \
         VALUES ($1, 'sandbox_validate', $2, $3, $4)",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(operator)
    .bind(detail)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Reads the generator version of the newest stored version, if there is one.
async fn stored_generator_version(
    conn: &mut PgConnection,
    slug: &str,
) -> Result<Option<String>, WriteError> {
    let version: Option<String> = sqlx::query_scalar(
        "SELECT generator_version FROM problem_versions WHERE problem_slug = $1 \
         ORDER BY version DESC LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(version)
}

/// Inserts the problem row if it is new, locks it, and refreshes its display copy.
async fn lock_problem(conn: &mut PgConnection, import: &StaticImport) -> Result<(), WriteError> {
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (slug) DO NOTHING",
    )
    .bind(&import.slug)
    .bind(&import.title)
    .bind(&import.track)
    .bind(&import.difficulty)
    .execute(&mut *conn)
    .await?;
    sqlx::query("SELECT slug FROM problems WHERE slug = $1 FOR UPDATE")
        .bind(&import.slug)
        .fetch_one(&mut *conn)
        .await?;
    sqlx::query(
        "UPDATE problems SET title = $2, track_slug = $3, difficulty = $4, updated_at = now() \
         WHERE slug = $1",
    )
    .bind(&import.slug)
    .bind(&import.title)
    .bind(&import.track)
    .bind(&import.difficulty)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Reads the newest stored version of `slug`, if there is one.
async fn latest_version(
    conn: &mut PgConnection,
    slug: &str,
) -> Result<Option<StoredVersion>, WriteError> {
    let row: Option<(i32, Vec<u8>, Value)> = sqlx::query_as(
        "SELECT version, content_sha256, manifest FROM problem_versions \
         WHERE problem_slug = $1 ORDER BY version DESC LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((version, hash_bytes, manifest)) = row else {
        return Ok(None);
    };
    let hash: [u8; 32] = hash_bytes
        .try_into()
        .map_err(|_| WriteError::CorruptStoredVersion {
            version,
            reason: "content hash is not 32 bytes",
        })?;
    let files: BTreeMap<String, String> =
        serde_json::from_value(manifest).map_err(|_| WriteError::CorruptStoredVersion {
            version,
            reason: "manifest is not a map of hex hashes",
        })?;
    Ok(Some(StoredVersion {
        version,
        hash,
        files: files.into_iter().collect(),
    }))
}

/// Inserts every row of one static version and its audit event.
async fn insert_static(
    conn: &mut PgConnection,
    import: &StaticImport,
    operator: &str,
) -> Result<(), WriteError> {
    insert_version(conn, import).await?;
    for (index, (input, expected)) in import.examples.iter().enumerate() {
        sqlx::query(
            "INSERT INTO problem_examples (problem_slug, version, ordinal, input, expected) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(ordinal(index))
        .bind(input)
        .bind(expected)
        .execute(&mut *conn)
        .await?;
    }
    for (lang, code) in &import.starters {
        sqlx::query(
            "INSERT INTO problem_starters (problem_slug, version, lang, code) VALUES ($1, $2, $3, $4)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(lang)
        .bind(code)
        .execute(&mut *conn)
        .await?;
    }
    for (rung, body) in &import.hints {
        sqlx::query(
            "INSERT INTO problem_hints (problem_slug, version, rung, body) VALUES ($1, $2, $3, $4)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(rung)
        .bind(body)
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query(
        "INSERT INTO reference_solutions (problem_slug, version, lang, code, sha256) \
         VALUES ($1, $2, 'python', $3, $4)",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(&import.reference_code)
    .bind(Sha256::digest(import.reference_code.as_bytes()).to_vec())
    .execute(&mut *conn)
    .await?;
    for ghost in &import.known_ghosts {
        sqlx::query(
            "INSERT INTO known_ghosts (problem_slug, version, ghost_id, kind, input, got, \
             exception_class, explanation, concepts) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(&import.slug)
        .bind(import.version)
        .bind(&ghost.ghost_id)
        .bind(&ghost.kind)
        .bind(&ghost.input)
        .bind(&ghost.got)
        .bind(&ghost.exception_class)
        .bind(&ghost.explanation)
        .bind(&ghost.concepts)
        .execute(&mut *conn)
        .await?;
    }
    replace_concepts(conn, import).await?;
    insert_event(conn, import, operator).await?;
    Ok(())
}

/// Inserts the `problem_versions` row. Its sandbox columns stay NULL until the one-time fill.
async fn insert_version(conn: &mut PgConnection, import: &StaticImport) -> Result<(), WriteError> {
    let manifest = Value::Object(
        import
            .manifest_public
            .iter()
            .map(|(path, hash)| (path.clone(), Value::String(hash.clone())))
            .collect::<Map<String, Value>>(),
    );
    let constraints = Value::Array(
        import
            .constraints
            .iter()
            .map(|item| Value::String(item.clone()))
            .collect(),
    );
    sqlx::query(
        "INSERT INTO problem_versions (problem_slug, version, content_sha256, manifest, title, \
         track_slug, difficulty, statement_markdown, constraints, signature, input_schema, \
         comparator, limits, reference_complexity, bench, complexity_note, engine_version, \
         generator_version, hidden_count) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(import.content_sha256.to_vec())
    .bind(manifest)
    .bind(&import.title)
    .bind(&import.track)
    .bind(&import.difficulty)
    .bind(&import.statement_markdown)
    .bind(constraints)
    .bind(&import.signature)
    .bind(&import.input_schema)
    .bind(&import.comparator)
    .bind(&import.limits)
    .bind(&import.reference_complexity)
    .bind(&import.bench)
    .bind(&import.complexity_note)
    .bind(&import.engine_version)
    .bind(&import.generator_version)
    .bind(import.hidden_count)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Replaces the concept tags for the problem. This is current metadata, not a version (D13).
async fn replace_concepts(
    conn: &mut PgConnection,
    import: &StaticImport,
) -> Result<(), WriteError> {
    sqlx::query("DELETE FROM problem_concepts WHERE problem_slug = $1")
        .bind(&import.slug)
        .execute(&mut *conn)
        .await?;
    for concept in &import.concepts {
        sqlx::query("INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ($1, $2)")
            .bind(&import.slug)
            .bind(concept)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Appends the `import` event. It holds counts and a hash prefix only (CAT-S6).
async fn insert_event(
    conn: &mut PgConnection,
    import: &StaticImport,
    operator: &str,
) -> Result<(), WriteError> {
    let prefix =
        import
            .content_sha256
            .iter()
            .take(6)
            .fold(String::with_capacity(12), |mut out, byte| {
                // Writing to a String cannot fail.
                let _ = write!(out, "{byte:02x}");
                out
            });
    let detail = serde_json::json!({
        "manifest_files": import.manifest_public.len(),
        "examples": import.examples.len(),
        "content_sha256_prefix": prefix,
    });
    sqlx::query(
        "INSERT INTO catalogue_events (problem_slug, action, version, operator, detail) \
         VALUES ($1, 'import', $2, $3, $4)",
    )
    .bind(&import.slug)
    .bind(import.version)
    .bind(operator)
    .bind(detail)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Converts a zero-based index into a 1-based ordinal. Saturates instead of panicking.
fn ordinal(index: usize) -> i32 {
    i32::try_from(index.saturating_add(1)).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(path, hash)| ((*path).to_string(), (*hash).to_string()))
            .collect()
    }

    #[test]
    fn diff_lists_added_removed_and_changed_paths_sorted() {
        let stored = pairs(&[("public/a", "h1"), ("public/b", "h2")]);
        let incoming = pairs(&[("public/b", "h2x"), ("public/c", "h3")]);
        let diff = diff_files(&stored, &incoming);
        assert_eq!(diff.added, vec!["public/c".to_string()]);
        assert_eq!(diff.removed, vec!["public/a".to_string()]);
        assert_eq!(diff.changed, vec!["public/b".to_string()]);
    }

    #[test]
    fn diff_of_identical_lists_is_empty() {
        let stored = pairs(&[("public/a", "h1")]);
        assert_eq!(diff_files(&stored, &stored), ManifestDiff::default());
    }
}
