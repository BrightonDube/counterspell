//! `counterspell-catalogue` — the problem-catalogue import CLI (C19, CAT-PT7).
//!
//! The operator runs this binary against the public `packs/` tree and a private
//! checkout (`--private-root`, CAT-PT2). It never fetches, clones or runs git,
//! never reads credential environment variables, and never executes pack code
//! itself — sandbox modes go through a `PackRunner`, and no runner is wired
//! into this build yet, so those paths report `SBX-07` and exit 2.
//!
//! Exit codes: 0 when every pack was created or unchanged, 1 for a refusal or
//! ordinary failure, 2 for usage errors, a missing runner, infrastructure
//! failures or a failed private-root preflight.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use counterspell_grimoire::meta::Meta;
use counterspell_grimoire::record::{self, RecordError};
use counterspell_judge::catalogue::approval::{check_approval, parse_approval};
use counterspell_judge::catalogue::discover::{
    classify, explicit_slug, Decision, DirEntry, SkipReason,
};
use counterspell_judge::catalogue::load::{assemble_import, assemble_sandbox, read_pack};
use counterspell_judge::catalogue::plan::{Plan, Refusal};
use counterspell_judge::catalogue::private_tree::{preflight, RootState};
use counterspell_judge::catalogue::report::{Outcome, PackReport};
use counterspell_judge::catalogue::retired::{parse_retired, RetiredFile};
use counterspell_judge::catalogue::sandbox_fill::{validate_sandbox, ValidateOutcome};
use counterspell_judge::catalogue::taxonomy::{parse_taxonomy, sync_taxonomy, Taxonomy};
use counterspell_judge::catalogue::write::{
    dry_run_static, write_static, StaticImport, WriteOutcome,
};
use serde::Deserialize;
use sqlx::PgPool;

/// Exit 0: everything created or unchanged.
const EXIT_OK: u8 = 0;
/// Exit 1: a pack refusal or ordinary failure.
const EXIT_REFUSED: u8 = 1;
/// Exit 2: usage errors, missing runner, infrastructure, failed preflight.
const EXIT_USAGE: u8 = 2;

/// The taxonomy file, outside every pack directory (D13).
const CATALOGUE_FILE: &str = "catalogue.toml";
/// The retired-values file at the private root (CAT-PT8).
const RETIRED_FILE: &str = "retired-values.toml";
/// Private packs live under `<private-root>/packs/<slug>/` (CAT-PT1).
const PRIVATE_PACKS_DIR: &str = "packs";

/// A catalogue verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    /// `import` — static or sandbox import over all packs (or `--slug`).
    Import,
    /// `validate-sandbox <slug>` — stamp the latest stored version (C18).
    ValidateSandbox,
    /// `publish <slug>` — implemented in C21.
    Publish,
    /// `unpublish <slug>` — implemented in C21.
    Unpublish,
    /// `list` — implemented in C22.
    List,
    /// `seed-dev` — implemented in C22.
    SeedDev,
    /// `seed-demo` — implemented in C22.
    SeedDemo,
}

impl Command {
    /// Parses the verb. Returns `None` for anything else.
    fn parse(verb: &str) -> Option<Self> {
        Some(match verb {
            "import" => Self::Import,
            "validate-sandbox" => Self::ValidateSandbox,
            "publish" => Self::Publish,
            "unpublish" => Self::Unpublish,
            "list" => Self::List,
            "seed-dev" => Self::SeedDev,
            "seed-demo" => Self::SeedDemo,
            _ => return None,
        })
    }

    /// The verb as typed, for messages.
    const fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::ValidateSandbox => "validate-sandbox",
            Self::Publish => "publish",
            Self::Unpublish => "unpublish",
            Self::List => "list",
            Self::SeedDev => "seed-dev",
            Self::SeedDemo => "seed-demo",
        }
    }
}

/// The parsed command line.
#[derive(Debug)]
struct Options {
    /// The verb.
    command: Command,
    /// `--packs`, default `packs`.
    packs_root: PathBuf,
    /// `--private-root` — required for verbs that read packs (CAT-PT2).
    private_root: Option<PathBuf>,
    /// The positional slug, or `--slug`.
    slug: Option<String>,
    /// `--dry-run` — plan only, a read-only transaction.
    dry_run: bool,
    /// `--static-only` or `--sandbox`; bare `import` means sandbox.
    mode: Mode,
    /// `--deny-warnings` — grimoire warnings refuse the pack.
    deny_warnings: bool,
    /// `--json` — JSON report lines.
    json: bool,
    /// `--version <n>` — `publish` pin (C21).
    version: Option<i32>,
}

/// The import mode flags, mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// No mode flag: the full sandbox import.
    Default,
    /// `--static-only`.
    StaticOnly,
    /// `--sandbox`.
    Sandbox,
}

/// A parse failure. Printed to stderr with the usage block; exits 2.
#[derive(Debug, PartialEq, Eq)]
struct UsageError(String);

/// Parses `argv[1..]` into [`Options`].
///
/// # Errors
///
/// Returns a [`UsageError`] for an unknown verb or flag, a missing flag value, a
/// missing or extra positional argument, or a `--version` that is not a number.
fn parse(args: &[String]) -> Result<Options, UsageError> {
    let mut iter = args.iter();
    let Some(verb) = iter.next() else {
        return Err(UsageError("a command is required".to_string()));
    };
    let command =
        Command::parse(verb).ok_or_else(|| UsageError(format!("unknown command `{verb}`")))?;

    let mut options = Options {
        command,
        packs_root: PathBuf::from("packs"),
        private_root: None,
        slug: None,
        dry_run: false,
        mode: Mode::Default,
        deny_warnings: false,
        json: false,
        version: None,
    };
    let mut positional: Vec<String> = Vec::new();
    let rest: Vec<String> = iter.cloned().collect();
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--packs" => {
                options.packs_root = PathBuf::from(flag_value(&rest, &mut index)?);
            }
            "--private-root" => {
                options.private_root = Some(PathBuf::from(flag_value(&rest, &mut index)?));
            }
            "--slug" => {
                options.slug = Some(flag_value(&rest, &mut index)?);
            }
            "--version" => {
                let raw = flag_value(&rest, &mut index)?;
                options.version =
                    Some(raw.parse::<i32>().map_err(|_| {
                        UsageError(format!("--version expects a number, got `{raw}`"))
                    })?);
            }
            "--dry-run" => options.dry_run = true,
            "--static-only" => {
                if options.mode == Mode::Sandbox {
                    return Err(UsageError(
                        "--static-only and --sandbox are mutually exclusive".to_string(),
                    ));
                }
                options.mode = Mode::StaticOnly;
            }
            "--sandbox" => {
                if options.mode == Mode::StaticOnly {
                    return Err(UsageError(
                        "--static-only and --sandbox are mutually exclusive".to_string(),
                    ));
                }
                options.mode = Mode::Sandbox;
            }
            "--deny-warnings" => options.deny_warnings = true,
            "--json" => options.json = true,
            other if other.starts_with('-') => {
                return Err(UsageError(format!("unknown flag `{other}`")));
            }
            other => positional.push(other.to_string()),
        }
        index += 1;
    }

    match command {
        Command::ValidateSandbox | Command::Publish | Command::Unpublish => {
            if positional.len() != 1 {
                return Err(UsageError(format!(
                    "`{}` takes exactly one slug",
                    command.name()
                )));
            }
            options.slug = Some(positional.remove(0));
        }
        _ if !positional.is_empty() => {
            return Err(UsageError(format!(
                "`{}` takes no positional arguments",
                command.name()
            )));
        }
        _ => {}
    }
    if options.dry_run && options.mode == Mode::Sandbox {
        return Err(UsageError("--dry-run never runs the sandbox".to_string()));
    }
    Ok(options)
}

/// Reads a `--flag value` pair, consuming the value slot.
fn flag_value(rest: &[String], index: &mut usize) -> Result<String, UsageError> {
    let flag = &rest[*index];
    *index += 1;
    rest.get(*index)
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| UsageError(format!("{flag} expects a value")))
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match parse(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("counterspell-catalogue: {}", error.0);
            eprintln!("{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(run(&options))
}

const USAGE: &str = "usage:\n  \
    counterspell-catalogue import [--slug <slug>] [--packs <dir>] --private-root <dir> \\\n    \
    [--static-only | --sandbox] [--dry-run] [--deny-warnings] [--json]\n  \
    counterspell-catalogue validate-sandbox <slug> [--packs <dir>] --private-root <dir>\n  \
    counterspell-catalogue publish <slug> [--version <n>]\n  \
    counterspell-catalogue unpublish <slug>\n  \
    counterspell-catalogue seed-dev --static-only --private-root <dir>\n  \
    counterspell-catalogue seed-demo --private-root <dir>\n  \
    counterspell-catalogue list";

/// Dispatches the parsed command.
async fn run(options: &Options) -> ExitCode {
    match options.command {
        Command::Import => run_import(options).await,
        Command::ValidateSandbox => run_validate(options).await,
        command => {
            eprintln!(
                "counterspell-catalogue: `{}` is implemented under C21/C22",
                command.name()
            );
            ExitCode::from(EXIT_USAGE)
        }
    }
}

/// The pre-database gate for every command that reads packs (CAT-PT2):
/// `--private-root` must be present and pass [`preflight`]. Runs before any
/// database connection.
fn pack_gate(options: &Options) -> Result<&Path, ExitCode> {
    let Some(private_root) = options.private_root.as_deref() else {
        eprintln!("counterspell-catalogue: --private-root is required (CAT-PT2)");
        return Err(ExitCode::from(EXIT_USAGE));
    };
    let state = root_state(private_root, &options.packs_root);
    if let Err(error) = preflight(state) {
        eprintln!(
            "counterspell-catalogue: private root refused: {} ({})",
            error.code(),
            error
        );
        return Err(ExitCode::from(EXIT_USAGE));
    }
    Ok(private_root)
}

/// Resolves the on-disk state of the private root for [`preflight`].
fn root_state(private_root: &Path, packs_root: &Path) -> RootState {
    let Ok(metadata) = fs::metadata(private_root) else {
        return RootState::Missing;
    };
    if !metadata.is_dir() || fs::read_dir(private_root).is_err() {
        return RootState::Unreadable;
    }
    let inside_public = match (private_root.canonicalize(), packs_root.canonicalize()) {
        (Ok(private), Ok(packs)) => {
            private.starts_with(&packs)
                || packs.parent().is_some_and(|repo| private.starts_with(repo))
        }
        _ => false,
    };
    RootState::Readable { inside_public }
}

/// Whether a sandbox runner is available. No production `PackRunner` is wired
/// into this build, so every sandbox path reports `SBX-07` and exits 2.
fn runner_available() -> bool {
    false
}

/// Reads `CATALOGUE_DATABASE_URL` and connects. The variable name is the only
/// thing ever printed; its value never is.
async fn connect() -> Result<PgPool, ExitCode> {
    let Ok(url) = env::var("CATALOGUE_DATABASE_URL") else {
        eprintln!("counterspell-catalogue: CATALOGUE_DATABASE_URL is not set");
        return Err(ExitCode::from(EXIT_USAGE));
    };
    PgPool::connect(&url).await.map_err(|error| {
        eprintln!("counterspell-catalogue: database connect failed: {error}");
        ExitCode::from(EXIT_USAGE)
    })
}

/// The operator label written to `catalogue_events` (default `$USER`).
fn operator() -> String {
    env::var("CATALOGUE_OPERATOR")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "operator".to_string())
}

/// `import` — the per-pack pipeline, then the report.
async fn run_import(options: &Options) -> ExitCode {
    let private_root = match pack_gate(options) {
        Ok(root) => root,
        Err(code) => return code,
    };
    let wants_sandbox =
        options.mode == Mode::Sandbox || (options.mode == Mode::Default && !options.dry_run);
    if wants_sandbox && !runner_available() {
        eprintln!(
            "counterspell-catalogue: SBX-07: no sandbox pack runner is wired into this build"
        );
        return ExitCode::from(EXIT_USAGE);
    }
    let pool = match connect().await {
        Ok(pool) => pool,
        Err(code) => return code,
    };
    let taxonomy = match read_taxonomy(&options.packs_root) {
        Ok(taxonomy) => taxonomy,
        Err(code) => {
            eprintln!("counterspell-catalogue: {code}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    if let Err(error) = sync_taxonomy(&pool, &taxonomy).await {
        eprintln!("counterspell-catalogue: taxonomy sync failed: {error}");
        return ExitCode::from(EXIT_USAGE);
    }

    let entries = match list_pack_entries(&options.packs_root) {
        Ok(entries) => entries,
        Err(code) => {
            eprintln!("counterspell-catalogue: {code}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let reports = import_packs(&pool, options, private_root, &taxonomy, &entries).await;
    print_reports(&reports, options.json);
    ExitCode::from(exit_code_for(&reports))
}

/// `validate-sandbox <slug>` — gate, fill and stamp through a `PackRunner` (C18).
///
/// No runner is wired into this build, so the command reports `SBX-07` and
/// exits 2 after the private-root preflight. The assembled path below is ready
/// for the runner when one lands.
async fn run_validate(options: &Options) -> ExitCode {
    let Ok(private_root) = pack_gate(options) else {
        return ExitCode::from(EXIT_USAGE);
    };
    if !runner_available() {
        eprintln!(
            "counterspell-catalogue: SBX-07: no sandbox pack runner is wired into this build"
        );
        return ExitCode::from(EXIT_USAGE);
    }
    let slug = options.slug.clone().unwrap_or_default();
    let pool = match connect().await {
        Ok(pool) => pool,
        Err(code) => return code,
    };
    let report = validate_one(&pool, &slug, options, private_root, "").await;
    print_reports(std::slice::from_ref(&report), options.json);
    match report.outcome {
        Outcome::Created | Outcome::Unchanged => ExitCode::from(EXIT_OK),
        Outcome::Refused(_) | Outcome::Failed(_) => ExitCode::from(EXIT_REFUSED),
    }
}

/// Runs the assembled `validate-sandbox` path for one slug. Unreachable while
/// [`runner_available`] is false — the runner and its pinned image digest land
/// with the sandbox-wiring task; the signature already takes them.
async fn validate_one(
    pool: &PgPool,
    slug: &str,
    options: &Options,
    private_root: &Path,
    image_digest: &str,
) -> PackReport {
    let public_dir = options.packs_root.join(slug);
    let private_dir = private_root.join(PRIVATE_PACKS_DIR).join(slug);
    if !private_dir.is_dir() {
        return refused(slug, "private_part_missing");
    }
    let Ok(files) = read_pack(&public_dir, Some(&private_dir)) else {
        return failed(slug, "pack_unreadable");
    };
    let Ok(manifest) = files.manifest() else {
        return refused(slug, "manifest_duplicate_path");
    };
    let record = match record::build(&public_dir) {
        Ok(record) => record,
        Err(RecordError::Invalid(_)) => return refused(slug, "static_errors"),
        Err(RecordError::Unreadable(_)) => return failed(slug, "pack_unreadable"),
    };
    let meta_text = match files.text_any("meta.toml") {
        Some(text) => text.to_string(),
        None => return refused(slug, "static_errors"),
    };
    let meta: Meta = match toml::from_str(&meta_text) {
        Ok(meta) => meta,
        Err(_) => return refused(slug, "meta_invalid"),
    };
    let Ok(retired) = read_retired(private_root) else {
        return failed(slug, "retired_unreadable");
    };
    let pack = match assemble_sandbox(&record, &meta, &files, &manifest) {
        Ok(pack) => pack,
        Err(error) => return refused(slug, error.code()),
    };
    let mut runner = unimplemented_runner();
    match validate_sandbox(
        pool,
        slug,
        &pack.sandbox_pack(slug, &retired, image_digest),
        runner.as_mut(),
        &operator(),
    )
    .await
    {
        Ok(ValidateOutcome::Validated { version }) => {
            report(slug, Outcome::Created, Some(version), None)
        }
        Ok(ValidateOutcome::AlreadyValidated { version }) => {
            refused_version(slug, "already_validated", version)
        }
        Ok(ValidateOutcome::Missing) => refused(slug, "version_missing"),
        Ok(ValidateOutcome::PrivateTreeChanged { version }) => {
            refused_version(slug, "private_tree_changed", version)
        }
        Ok(ValidateOutcome::Refused(refusal)) => refused(slug, refusal.code()),
        Err(_) => failed(slug, "validate_error"),
    }
}

/// The runner placeholder for the wired-but-unreachable sandbox path.
fn unimplemented_runner() -> Box<dyn counterspell_grimoire::runner::PackRunner> {
    unimplemented!("no PackRunner is wired into this build")
}

/// Reads and parses `packs/catalogue.toml` (D13). The taxonomy is shared
/// catalogue state, so a missing or malformed file is infrastructure, not a
/// pack refusal.
fn read_taxonomy(packs_root: &Path) -> Result<Taxonomy, String> {
    let path = packs_root.join(CATALOGUE_FILE);
    let text = fs::read_to_string(&path)
        .map_err(|_| format!("{CATALOGUE_FILE} is missing or unreadable"))?;
    parse_taxonomy(&text).map_err(|error| format!("{CATALOGUE_FILE} is invalid: {error:?}"))
}

/// Reads `<private-root>/retired-values.toml`; an absent file means no retired
/// values (rotation checks still apply at fill time).
fn read_retired(private_root: &Path) -> Result<RetiredFile, ()> {
    let path = private_root.join(RETIRED_FILE);
    match fs::read_to_string(path) {
        Ok(text) => parse_retired(&text).map_err(|_| ()),
        Err(_) => parse_retired("").map_err(|_| ()),
    }
}

/// The directory names directly under the public packs root.
fn list_pack_entries(packs_root: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = Vec::new();
    let read = fs::read_dir(packs_root)
        .map_err(|_| format!("packs root `{}` is unreadable", packs_root.display()))?;
    for entry in read {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// The `status` field of one pack's `meta.toml`, and nothing else (CAT-I3).
#[derive(Deserialize)]
struct StatusProbe {
    /// `draft` or `published`; anything else is not `published`.
    status: Option<String>,
}

/// Reads just `meta.toml`'s `status`, so the draft gate does not depend on the
/// rest of the file parsing.
fn read_status(public_dir: &Path) -> Option<String> {
    let text = fs::read_to_string(public_dir.join("meta.toml")).ok()?;
    toml::from_str::<StatusProbe>(&text)
        .ok()
        .and_then(|probe| probe.status)
}

/// Runs the import pipeline for each selected pack and each orphan private
/// directory.
async fn import_packs(
    pool: &PgPool,
    options: &Options,
    private_root: &Path,
    taxonomy: &Taxonomy,
    entries: &[String],
) -> Vec<PackReport> {
    let private_packs = private_root.join(PRIVATE_PACKS_DIR);
    let private_slugs: Vec<String> = fs::read_dir(&private_packs)
        .map(|read| {
            read.filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
                // `_` marks test and dogfood dirs in both trees; never orphans.
                .filter(|name| !name.starts_with('_'))
                .collect()
        })
        .unwrap_or_default();

    let selected: Vec<String> = match &options.slug {
        Some(slug) => match explicit_slug(slug) {
            Ok(()) => vec![slug.clone()],
            Err(_) => return vec![refused(slug, "underscore_pack")],
        },
        None => entries.to_vec(),
    };

    let mut reports: Vec<PackReport> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for slug in &selected {
        let public_dir = options.packs_root.join(slug);
        let is_packish = matches!(
            classify(&DirEntry {
                name: slug,
                is_dir: public_dir.is_dir(),
                has_meta: public_dir.join("meta.toml").is_file(),
                // The draft gate runs after the join; status is withheld here so
                // any real pack dir classifies as Skip(Draft), not skipped.
                status: None,
            }),
            Decision::Skip(SkipReason::Draft) | Decision::Import
        );
        if !is_packish {
            // Underscore names, files and meta-less directories are skipped
            // silently during a scan; an explicit --slug reports them.
            if options.slug.is_some() {
                reports.push(refused(slug, "not_a_pack"));
            }
            continue;
        }
        seen.push(slug);
        let private_dir = private_packs.join(slug);
        if !private_dir.is_dir() {
            reports.push(refused(slug, "private_part_missing"));
            continue;
        }
        if read_status(&public_dir).as_deref() != Some("published") {
            reports.push(refused(slug, "draft_pack"));
            continue;
        }
        reports.push(import_one(pool, options, slug, &public_dir, &private_dir, taxonomy).await);
    }
    for slug in private_slugs
        .iter()
        .filter(|slug| !seen.contains(&slug.as_str()))
    {
        reports.push(refused(slug, "orphan_private_pack"));
    }
    reports
}

/// Runs one paired pack through validation, assembly and the plan or write.
async fn import_one(
    pool: &PgPool,
    options: &Options,
    slug: &str,
    public_dir: &Path,
    private_dir: &Path,
    taxonomy: &Taxonomy,
) -> PackReport {
    let Ok(report) = counterspell_grimoire::validate_static(public_dir) else {
        return failed(slug, "pack_unreadable");
    };
    if report.has_errors() {
        return refused(slug, "static_errors");
    }
    if options.deny_warnings && report.warning_count() > 0 {
        return refused(slug, "static_warnings");
    }

    let record = match record::build(public_dir) {
        Ok(record) => record,
        Err(RecordError::Invalid(_)) => return refused(slug, "static_errors"),
        Err(RecordError::Unreadable(_)) => return failed(slug, "pack_unreadable"),
    };
    if !taxonomy
        .tracks
        .iter()
        .any(|entry| entry.slug == record.track)
        || !record
            .concepts
            .iter()
            .all(|concept| taxonomy.concepts.iter().any(|entry| &entry.slug == concept))
    {
        return refused(slug, "unknown_taxonomy");
    }

    let Ok(files) = read_pack(public_dir, Some(private_dir)) else {
        return failed(slug, "pack_unreadable");
    };
    let Ok(manifest) = files.manifest() else {
        return refused(slug, "manifest_duplicate_path");
    };
    let meta_text = match files.text_any("meta.toml") {
        Some(text) => text.to_string(),
        None => return refused(slug, "static_errors"),
    };
    let meta: Meta = match toml::from_str(&meta_text) {
        Ok(meta) => meta,
        Err(_) => return refused(slug, "meta_invalid"),
    };

    let approval = files
        .approval_text()
        .and_then(|text| parse_approval(text).ok());
    let version = i32::try_from(record.version).unwrap_or(i32::MAX);
    if let Err(refusal) = check_approval(
        approval.as_ref(),
        slug,
        version,
        &manifest.content_sha256_hex(),
    ) {
        return refused(slug, &refusal.to_string());
    }

    let import = match assemble_import(&record, &meta, &files, &manifest) {
        Ok(import) => import,
        Err(error) => return refused(slug, error.code()),
    };
    write_or_plan(pool, options, slug, &import).await
}

/// Plans the import read-only under `--dry-run`, or writes it otherwise.
async fn write_or_plan(
    pool: &PgPool,
    options: &Options,
    slug: &str,
    import: &StaticImport,
) -> PackReport {
    if options.dry_run {
        return match dry_run_static(pool, import).await {
            Ok(report) => {
                let outcome = match report.plan {
                    Plan::Create => Outcome::Created,
                    Plan::Unchanged => Outcome::Unchanged,
                    Plan::Refuse(refusal) => Outcome::Refused(refusal_code(&refusal)),
                };
                let mut counts = std::collections::BTreeMap::new();
                counts.insert("added", report.diff.added.len());
                counts.insert("removed", report.diff.removed.len());
                counts.insert("changed", report.diff.changed.len());
                PackReport {
                    slug: slug.to_string(),
                    version: Some(import.version),
                    outcome,
                    counts,
                    hash_hex: Some(hex(&import.content_sha256)),
                }
            }
            Err(_) => failed(slug, "plan_read_failed"),
        };
    }
    match write_static(pool, import, &operator()).await {
        Ok(WriteOutcome::Created { version }) => report(
            slug,
            Outcome::Created,
            Some(version),
            Some(&import.content_sha256),
        ),
        Ok(WriteOutcome::Unchanged { version }) => report(
            slug,
            Outcome::Unchanged,
            Some(version),
            Some(&import.content_sha256),
        ),
        Ok(WriteOutcome::Refused(refusal)) => refused(slug, &refusal_code(&refusal)),
        Err(_) => failed(slug, "write_failed"),
    }
}

/// The stable code for a plan refusal.
fn refusal_code(refusal: &Refusal) -> String {
    match refusal {
        Refusal::VersionNotBumped { .. } => "version_not_bumped".to_string(),
        Refusal::VersionRegressed => "version_regressed".to_string(),
    }
}

/// A refusal report with no version.
fn refused(slug: &str, code: &str) -> PackReport {
    report(slug, Outcome::Refused(code.to_string()), None, None)
}

/// A refusal report carrying the stored version.
fn refused_version(slug: &str, code: &str, version: i32) -> PackReport {
    report(
        slug,
        Outcome::Refused(code.to_string()),
        Some(version),
        None,
    )
}

/// An ordinary-failure report.
fn failed(slug: &str, code: &str) -> PackReport {
    report(slug, Outcome::Failed(code.to_string()), None, None)
}

/// Builds a `PackReport` with empty counts.
fn report(
    slug: &str,
    outcome: Outcome,
    version: Option<i32>,
    hash: Option<&[u8; 32]>,
) -> PackReport {
    PackReport {
        slug: slug.to_string(),
        version,
        outcome,
        counts: std::collections::BTreeMap::new(),
        hash_hex: hash.map(hex),
    }
}

/// Lower-case hex of a 32-byte digest. Only the renderer's prefix ever prints.
fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// Prints the reports, one line or one JSON object per pack. Hidden values and
/// private file contents never appear — only slugs, codes, counts and the hash
/// prefix the renderer already truncates.
fn print_reports(reports: &[PackReport], json_mode: bool) {
    for report in reports {
        if json_mode {
            println!("{}", report.render_json());
        } else {
            println!("{}", report.render_text());
        }
    }
}

/// The exit code for a set of pack reports: 0 only when every pack was created
/// or unchanged.
fn exit_code_for(reports: &[PackReport]) -> u8 {
    let clean = reports
        .iter()
        .all(|report| matches!(report.outcome, Outcome::Created | Outcome::Unchanged));
    if clean {
        EXIT_OK
    } else {
        EXIT_REFUSED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_import_flags() {
        let options = parse(&args(&[
            "import",
            "--packs",
            "packs",
            "--private-root",
            "/private",
            "--static-only",
            "--dry-run",
            "--deny-warnings",
            "--json",
        ]))
        .expect("parse");
        assert_eq!(options.command, Command::Import);
        assert_eq!(options.packs_root, PathBuf::from("packs"));
        assert_eq!(options.private_root, Some(PathBuf::from("/private")));
        assert_eq!(options.mode, Mode::StaticOnly);
        assert!(options.dry_run && options.deny_warnings && options.json);
    }

    #[test]
    fn parses_validate_sandbox_slug() {
        let options = parse(&args(&[
            "validate-sandbox",
            "count-pairs",
            "--private-root",
            "/private",
        ]))
        .expect("parse");
        assert_eq!(options.command, Command::ValidateSandbox);
        assert_eq!(options.slug.as_deref(), Some("count-pairs"));
    }

    #[test]
    fn rejects_unknown_verb() {
        assert!(parse(&args(&["frobnicate"])).is_err());
    }

    #[test]
    fn rejects_extra_positional() {
        assert!(parse(&args(&["import", "count-pairs"])).is_err());
    }

    #[test]
    fn rejects_missing_slug() {
        assert!(parse(&args(&["validate-sandbox"])).is_err());
    }

    #[test]
    fn rejects_flag_without_value() {
        assert!(parse(&args(&["import", "--private-root"])).is_err());
    }

    #[test]
    fn rejects_mode_conflict() {
        assert!(parse(&args(&["import", "--static-only", "--sandbox",])).is_err());
        assert!(parse(&args(&["import", "--dry-run", "--sandbox"])).is_err());
    }

    #[test]
    fn missing_private_root_exits_two() {
        // pack_gate only ever fails with an EXIT_USAGE code.
        let options = parse(&args(&["import", "--static-only"])).expect("parse");
        assert!(pack_gate(&options).is_err());
    }

    #[test]
    fn missing_root_dir_exits_two() {
        let options = parse(&args(&[
            "import",
            "--static-only",
            "--private-root",
            "/definitely/not/a/real/path",
        ]))
        .expect("parse");
        assert!(pack_gate(&options).is_err());
    }

    #[test]
    fn exit_mapping() {
        assert_eq!(exit_code_for(&[]), EXIT_OK);
        assert_eq!(
            exit_code_for(&[report("a", Outcome::Created, None, None)]),
            EXIT_OK
        );
        assert_eq!(exit_code_for(&[refused("a", "draft_pack")]), EXIT_REFUSED);
    }

    #[test]
    fn unimplemented_verbs_parse() {
        for verb in ["publish", "unpublish", "list", "seed-dev", "seed-demo"] {
            let parsed = match verb {
                "publish" | "unpublish" => parse(&args(&[verb, "slug"])),
                _ => parse(&args(&[verb])),
            };
            assert!(parsed.is_ok(), "{verb} should parse");
        }
    }
}
