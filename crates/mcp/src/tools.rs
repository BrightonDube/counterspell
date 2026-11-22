//! The six tool handlers. Each one takes typed arguments and returns bounded JSON or a
//! `ToolError`. The rmcp layer in `server.rs` only translates to and from protocol types.
//!
//! The first three tools never run code. The last three use the runner slot, which
//! fails closed when no sandbox runner is configured.

use crate::config::ConfigError;
use crate::errors::{bounded, ErrorCategory, ToolError};
use crate::judge_adapter::{load_pack, PackContext};
use crate::paths::{private_pack_dir, resolve_inside, PathError};
use crate::runner::{PackJob, RunnerSlot, SlotError};
use crate::schemas::{
    ExplainVerdictArgs, FindGhostArgs, FitComplexityArgs, RunReferenceArgs, ShrinkInputArgs,
    ValidatePackArgs, MAX_FIT_POINTS,
};
use counterspell_engine::complexity::{self, Class, FitConfig, Mode, Point};
use counterspell_engine::{
    build_autopsy, classify, hunt, AbortReason, Budget, Classification, FailureKind,
    FailurePredicate, HuntOutcome, ProgramResult, SystemClock,
};
use counterspell_grimoire::record::{self, signature_descriptor};
use counterspell_grimoire::runner::{ProgramOracle, RunnerError, Target};
use counterspell_grimoire::{validate_static, validate_static_pair};
use counterspell_judge::sandbox::pack_runner::REFERENCE_SOURCE;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The time an explanation assumes for an observed timeout, in milliseconds.
const EXPLAIN_LIMIT_MS: u64 = 2000;

/// Maps a path refusal to a tool error.
fn path_error(error: &PathError, root: &Path) -> ToolError {
    let (category, message) = match error {
        PathError::Invalid(reason) => (ErrorCategory::InvalidRequest, *reason),
        PathError::Symlink => (
            ErrorCategory::InvalidRequest,
            "the path passes through a symbolic link",
        ),
        PathError::NotFound => (ErrorCategory::NotFound, "the path does not exist"),
        PathError::Outside => (
            ErrorCategory::InvalidRequest,
            "the path resolves outside the workspace",
        ),
    };
    ToolError::new(category, message, root)
}

/// Maps a sandbox configuration refusal to a tool error. The category never reads as a verdict.
fn config_error(error: &ConfigError, root: &Path) -> ToolError {
    ToolError::new(
        ErrorCategory::SandboxNotConfigured,
        &error.to_string(),
        root,
    )
}

/// Resolves a pack directory argument and loads the pack.
fn pack_from(root: &Path, pack_path: &str) -> Result<(PathBuf, PackContext), ToolError> {
    let pack_dir = resolve_inside(root, pack_path).map_err(|error| path_error(&error, root))?;
    let context = load_pack(&pack_dir, root)?;
    Ok((pack_dir, context))
}

/// Resolves an incorrect program named inside the pack and reads its bytes.
fn program_from(pack_dir: &Path, root: &Path, program: &str) -> Result<Vec<u8>, ToolError> {
    let is_python = Path::new(program)
        .extension()
        .is_some_and(|extension| extension == "py");
    if !program.starts_with("incorrect/") || !is_python {
        return Err(ToolError::new(
            ErrorCategory::InvalidRequest,
            "the program must be a .py file under incorrect/",
            root,
        ));
    }
    // `incorrect/` lives in the private tree since the pack split; try it
    // first, falling back to the public tree for packs that predate it.
    let private_path =
        private_pack_dir(pack_dir).and_then(|dir| resolve_inside(&dir, program).ok());
    let path = private_path
        .or_else(|| resolve_inside(pack_dir, program).ok())
        .ok_or_else(|| path_error(&PathError::Outside, root))?;
    fs::read(path).map_err(|_| {
        ToolError::new(
            ErrorCategory::NotFound,
            "the program could not be read",
            root,
        )
    })
}

/// Checks an input against the pack schema before anything runs.
fn check_input(context: &PackContext, input: &Value, root: &Path) -> Result<(), ToolError> {
    context
        .meta
        .input_schema
        .contains(input)
        .map_err(|violation| {
            ToolError::new(
                ErrorCategory::InvalidRequest,
                &format!(
                    "input is outside the pack schema at {}: {}",
                    violation.path, violation.reason
                ),
                root,
            )
        })
}

/// `validate_pack`: the static Grimoire report. Never runs code.
///
/// `reference/` and `incorrect/` live in the private pack tree (the
/// `counterspell-packs-private` checkout), so the tool validates the merged
/// pack when `COUNTERSPELL_PACKS_PRIVATE_ROOT` names one.
///
/// # Errors
///
/// Returns `InvalidRequest` or `NotFound` for a bad path.
pub fn validate_pack(root: &Path, args: &ValidatePackArgs) -> Result<Value, ToolError> {
    let pack_dir =
        resolve_inside(root, &args.pack_path).map_err(|error| path_error(&error, root))?;
    let private_dir = private_pack_dir(&pack_dir);
    let report = match &private_dir {
        Some(private_dir) => validate_static_pair(&pack_dir, private_dir),
        None => validate_static(&pack_dir),
    }
    .map_err(|_| ToolError::new(ErrorCategory::Internal, "the pack could not be read", root))?;
    let warnings_block = args.deny_warnings && report.warning_count() > 0;
    Ok(json!({
        "passed": !report.has_errors() && !warnings_block,
        "report": report.to_json(),
    }))
}

/// `explain_verdict`: the engine's Bug Autopsy for one observed failure. Never runs code.
///
/// # Errors
///
/// Returns `InvalidRequest` for an unknown kind or a bad path.
pub fn explain_verdict(root: &Path, args: &ExplainVerdictArgs) -> Result<Value, ToolError> {
    let (_, context) = pack_from(root, &args.pack_path)?;
    let (predicate, got) = match args.kind.as_str() {
        "wrong" => {
            let actual = args.actual.clone().ok_or_else(|| {
                ToolError::new(
                    ErrorCategory::InvalidRequest,
                    "a wrong answer needs actual",
                    root,
                )
            })?;
            (
                FailurePredicate {
                    kind: FailureKind::WrongAnswer,
                    exception_class: None,
                },
                ProgramResult::Returned(actual),
            )
        }
        "crash" => {
            let class = args.exception_class.clone().unwrap_or_default();
            (
                FailurePredicate {
                    kind: FailureKind::RuntimeError,
                    exception_class: Some(class.clone()),
                },
                ProgramResult::Crashed {
                    exception_class: class,
                    message: String::new(),
                },
            )
        }
        "timeout" => (
            FailurePredicate {
                kind: FailureKind::Timeout,
                exception_class: None,
            },
            ProgramResult::TimedOut,
        ),
        _ => {
            return Err(ToolError::new(
                ErrorCategory::InvalidRequest,
                "kind must be wrong, crash or timeout",
                root,
            ))
        }
    };
    let known: Vec<_> = context
        .known
        .iter()
        .map(|ghost| ghost.known.clone())
        .collect();
    let autopsy = build_autopsy(
        &args.input,
        &predicate,
        args.expected.as_ref(),
        &got,
        Duration::from_millis(EXPLAIN_LIMIT_MS),
        &known,
        &context.meta.concepts,
    );
    serde_json::to_value(&autopsy).map_err(|_| {
        ToolError::new(
            ErrorCategory::Internal,
            "the autopsy could not be encoded",
            root,
        )
    })
}

/// Maps a declared class name to the engine class. Only the seven launch names are accepted.
#[must_use]
pub fn parse_class(name: &str) -> Option<Class> {
    match name.trim() {
        "1" => Some(Class::Constant),
        "log n" => Some(Class::Logarithmic),
        "n" => Some(Class::Linear),
        "n log n" => Some(Class::Linearithmic),
        "n^2" => Some(Class::Quadratic),
        "n^3" => Some(Class::Cubic),
        "2^n" => Some(Class::Exponential),
        _ => None,
    }
}

/// `fit_complexity`: the pure engine fit of supplied timings. Never runs code.
///
/// # Errors
///
/// Returns `InvalidRequest` for too many samples, an unknown class, or samples the fit rejects.
pub fn fit_complexity(root: &Path, args: &FitComplexityArgs) -> Result<Value, ToolError> {
    if args.samples.len() > MAX_FIT_POINTS {
        return Err(ToolError::new(
            ErrorCategory::InvalidRequest,
            "too many timing samples",
            root,
        ));
    }
    let reference = parse_class(&args.reference_complexity).ok_or_else(|| {
        ToolError::new(
            ErrorCategory::InvalidRequest,
            "reference_complexity must be one of 1, log n, n, n log n, n^2, n^3, 2^n",
            root,
        )
    })?;
    let points: Vec<Point> = args
        .samples
        .iter()
        .map(|sample| Point {
            n: sample.n,
            median_ns: sample.median_ns,
            spread: None,
        })
        .collect();
    let report = complexity::fit(&points, Mode::Full, &FitConfig::default())
        .map_err(|error| ToolError::new(ErrorCategory::InvalidRequest, &error.to_string(), root))?;
    let comparison = complexity::compare(&report, reference);
    Ok(json!({
        "report": serde_json::to_value(&report).unwrap_or(Value::Null),
        "reference_complexity": args.reference_complexity,
        "gap": comparison.gap,
        "reference_plausible": comparison.reference_plausible,
    }))
}

/// Encodes one raw program result as bounded JSON.
fn result_json(result: &ProgramResult) -> Value {
    match result {
        ProgramResult::Returned(value) => json!({ "outcome": "returned", "value": value }),
        ProgramResult::Crashed {
            exception_class,
            message,
        } => json!({
            "outcome": "crashed",
            "exception_class": exception_class,
            "message": bounded(message, 240),
        }),
        ProgramResult::TimedOut => json!({ "outcome": "timed_out" }),
        ProgramResult::Garbled { snippet } => json!({
            "outcome": "garbled",
            "snippet": bounded(snippet, 240),
        }),
    }
}

/// Maps a runner fault to an infrastructure error. It is never a user verdict.
fn runner_fault(error: &RunnerError, root: &Path) -> ToolError {
    let RunnerError::Infrastructure(message) = error;
    ToolError::new(ErrorCategory::SandboxInfrastructure, message, root)
}

/// Maps a slot failure to the tool error category. A factory fault is infrastructure, never a
/// verdict, and a missing sandbox fails closed.
fn slot_error(error: SlotError, root: &Path) -> ToolError {
    match error {
        SlotError::Config(error) => config_error(&error, root),
        SlotError::Job(error) => error,
        SlotError::Runner(error) => runner_fault(&error, root),
    }
}

/// The path of the trusted reference inside a pack.
const REFERENCE_FILE: &str = "reference/solution.py";

/// Builds what a factory slot needs for one call: the reference, the one program under test (if
/// any) and the driver-protocol signature. Nothing else from the pack goes to the sandbox side.
fn pack_job(
    pack_dir: &Path,
    root: &Path,
    context: &PackContext,
    program: Option<(&str, &[u8])>,
) -> Result<PackJob, ToolError> {
    let mut sources = BTreeMap::new();
    let reference = context.pack.text(REFERENCE_FILE).ok_or_else(|| {
        ToolError::new(
            ErrorCategory::NotFound,
            "the pack has no readable reference solution",
            root,
        )
    })?;
    sources.insert(REFERENCE_SOURCE.to_owned(), reference.to_owned());
    if let Some((path, bytes)) = program {
        let text = String::from_utf8(bytes.to_vec()).map_err(|_| {
            ToolError::new(
                ErrorCategory::InvalidRequest,
                "the program is not valid UTF-8 text",
                root,
            )
        })?;
        sources.insert(path.to_owned(), text);
    }
    let record = match private_pack_dir(pack_dir) {
        Some(private_dir) => record::build_pair(pack_dir, &private_dir),
        None => record::build(pack_dir),
    }
    .map_err(|error| {
        let detail = match &error {
            record::RecordError::Invalid(report) => report
                .diagnostics()
                .iter()
                .take(3)
                .map(counterspell_grimoire::report::Diagnostic::to_text_line)
                .collect::<Vec<_>>()
                .join("; "),
            record::RecordError::Unreadable(_) => "the pack could not be read".to_owned(),
        };
        ToolError::new(
            ErrorCategory::InvalidRequest,
            &format!("the pack does not pass static validation: {detail}"),
            root,
        )
    })?;
    let signature = signature_descriptor(&record.signature).map_err(|kind| {
        ToolError::new(
            ErrorCategory::InvalidRequest,
            &format!("the signature uses an unsupported type: {kind}"),
            root,
        )
    })?;
    Ok(PackJob { sources, signature })
}

/// `run_reference`: runs the trusted reference on one input through the sandbox runner, then
/// releases the runner so no sandbox container stays open after the call.
///
/// # Errors
///
/// Returns `SandboxNotConfigured` without a runner, `SandboxInfrastructure` on a runner fault,
/// and `InvalidRequest` for an input outside the schema.
pub fn run_reference(
    root: &Path,
    slot: &mut RunnerSlot,
    args: &RunReferenceArgs,
) -> Result<Value, ToolError> {
    let result = run_reference_inner(root, slot, args);
    slot.release();
    result
}

/// `run_reference`: runs the trusted reference on one input through the sandbox runner.
///
/// # Errors
///
/// Returns `SandboxNotConfigured` without a runner, `SandboxInfrastructure` on a runner fault,
/// and `InvalidRequest` for an input outside the schema.
fn run_reference_inner(
    root: &Path,
    slot: &mut RunnerSlot,
    args: &RunReferenceArgs,
) -> Result<Value, ToolError> {
    let (pack_dir, context) = pack_from(root, &args.pack_path)?;
    check_input(&context, &args.input, root)?;
    let limit = limit_of(&context, root)?;
    let runner = slot
        .get_for(|| pack_job(&pack_dir, root, &context, None))
        .map_err(|error| slot_error(error, root))?;
    let result = runner
        .run(Target::Reference, &args.input, limit)
        .map_err(|error| runner_fault(&error, root))?;
    Ok(result_json(&result))
}

/// The per-test time limit of a pack, from its metadata.
fn limit_of(context: &PackContext, root: &Path) -> Result<Duration, ToolError> {
    let millis = u64::try_from(context.meta.limits.time_ms).map_err(|_| {
        ToolError::new(
            ErrorCategory::InvalidRequest,
            "the pack time limit is invalid",
            root,
        )
    })?;
    Ok(Duration::from_millis(millis))
}

/// Builds the budget from optional arguments, starting from the engine defaults.
fn budget_from(
    args: Option<&crate::schemas::BudgetArgs>,
    root: &Path,
) -> Result<Budget, ToolError> {
    let mut budget = Budget::default();
    if let Some(args) = args {
        if let Some(value) = args.max_candidates {
            budget.max_candidates = value;
        }
        if let Some(value) = args.search_ms {
            budget.search_wall = Duration::from_millis(value);
        }
        if let Some(value) = args.max_shrink_evals {
            budget.max_shrink_evals = value;
        }
        if let Some(value) = args.shrink_ms {
            budget.shrink_wall = Duration::from_millis(value);
        }
        if let Some(value) = args.total_ms {
            budget.total_wall = Duration::from_millis(value);
        }
    }
    budget
        .validate()
        .map_err(|reason| ToolError::new(ErrorCategory::InvalidRequest, reason, root))?;
    Ok(budget)
}

/// `find_ghost_input`: the engine hunt, with every evaluation run through the sandbox runner. The
/// runner is released when the hunt ends, whatever the result.
///
/// # Errors
///
/// Returns `SandboxNotConfigured` without a runner, `SandboxInfrastructure` when the
/// runner fails, `EngineAborted` for other aborts, and `InvalidRequest` for bad arguments.
/// A search that finds nothing within budget is a normal result.
pub fn find_ghost_input(
    root: &Path,
    slot: &mut RunnerSlot,
    image_digest: &str,
    args: &FindGhostArgs,
) -> Result<Value, ToolError> {
    let result = find_ghost_input_inner(root, slot, image_digest, args);
    slot.release();
    result
}

/// `find_ghost_input`: the engine hunt, with every evaluation run through the sandbox runner.
///
/// `image_digest` is the pinned Python image, resolved once at start-up from the language catalogue.
///
/// # Errors
///
/// Returns `SandboxNotConfigured` without a runner, `SandboxInfrastructure` when the
/// runner fails, `EngineAborted` for other aborts, and `InvalidRequest` for bad arguments.
/// A search that finds nothing within budget is a normal result.
fn find_ghost_input_inner(
    root: &Path,
    slot: &mut RunnerSlot,
    image_digest: &str,
    args: &FindGhostArgs,
) -> Result<Value, ToolError> {
    let (pack_dir, context) = pack_from(root, &args.pack_path)?;
    let program_bytes = program_from(&pack_dir, root, &args.program)?;
    let budget = budget_from(args.budget.as_ref(), root)?;
    let runner = slot
        .get_for(|| {
            pack_job(
                &pack_dir,
                root,
                &context,
                Some((&args.program, &program_bytes)),
            )
        })
        .map_err(|error| slot_error(error, root))?;
    let mut request = context.hunt_request(&program_bytes, image_digest);
    request.budget = budget;
    request.seed_override = args.seed_override;
    let mut oracle = ProgramOracle::new(runner, &args.program);
    let report = hunt(&request, &mut oracle, &SystemClock::new());
    match &report.outcome {
        HuntOutcome::Found(_) | HuntOutcome::NotFound(_) => {
            serde_json::to_value(&report).map_err(|_| {
                ToolError::new(
                    ErrorCategory::Internal,
                    "the hunt could not be encoded",
                    root,
                )
            })
        }
        HuntOutcome::Aborted(reason) => Err(abort_error(reason, root)),
    }
}

/// Maps an engine abort to the tool error category.
fn abort_error(reason: &AbortReason, root: &Path) -> ToolError {
    match reason {
        AbortReason::OracleFailure(message) => {
            ToolError::new(ErrorCategory::SandboxInfrastructure, message, root)
        }
        AbortReason::ReferenceUnreliable => ToolError::new(
            ErrorCategory::EngineAborted,
            "the reference failed too often, so its expected values cannot be trusted",
            root,
        ),
        AbortReason::InvalidRequest(message) => {
            ToolError::new(ErrorCategory::InvalidRequest, message, root)
        }
    }
}

/// `shrink_input`: the engine shrinker, where every probe is judged by the sandbox runner. The
/// runner is released when the shrink ends, whatever the result.
///
/// # Errors
///
/// Returns `InvalidRequest` for a timeout target, a bad input or a bad path,
/// `SandboxNotConfigured` without a runner, `SandboxInfrastructure` on a runner fault, and
/// `EngineAborted` when the shrunk input does not reproduce the failure.
pub fn shrink_input(
    root: &Path,
    slot: &mut RunnerSlot,
    args: &ShrinkInputArgs,
) -> Result<Value, ToolError> {
    let result = shrink_input_inner(root, slot, args);
    slot.release();
    result
}

/// `shrink_input`: the engine shrinker, where every probe is judged by the sandbox runner.
///
/// # Errors
///
/// Returns `InvalidRequest` for a timeout target, a bad input or a bad path,
/// `SandboxNotConfigured` without a runner, `SandboxInfrastructure` on a runner fault, and
/// `EngineAborted` when the shrunk input does not reproduce the failure.
fn shrink_input_inner(
    root: &Path,
    slot: &mut RunnerSlot,
    args: &ShrinkInputArgs,
) -> Result<Value, ToolError> {
    let (pack_dir, context) = pack_from(root, &args.pack_path)?;
    let program_bytes = program_from(&pack_dir, root, &args.program)?;
    check_input(&context, &args.input, root)?;
    let kind = match args.target.kind.as_str() {
        "wrong" => FailureKind::WrongAnswer,
        "crash" => FailureKind::RuntimeError,
        _ => {
            return Err(ToolError::new(
                ErrorCategory::InvalidRequest,
                "shrink_input supports only wrong and crash targets",
                root,
            ))
        }
    };
    let limit = limit_of(&context, root)?;
    let comparator = context.meta.engine_comparator();
    let runner = slot
        .get_for(|| {
            pack_job(
                &pack_dir,
                root,
                &context,
                Some((&args.program, &program_bytes)),
            )
        })
        .map_err(|error| slot_error(error, root))?;
    let mut oracle = ProgramOracle::new(runner, &args.program);
    let wall = Duration::from_millis(args.wall_ms);
    let started = Instant::now();
    let mut evaluate = |candidate: &Value, eval_limit: Duration| {
        counterspell_engine::Oracle::evaluate(&mut oracle, candidate, eval_limit)
    };
    let mut keeps = |observation: &counterspell_engine::Observation| {
        matches!(
            classify(observation, &comparator),
            Classification::Fail(predicate) if predicate.kind == kind
        )
    };
    let mut now = move || started.elapsed();
    let outcome = counterspell_engine::shrink::Driver::new(
        &context.meta.input_schema,
        &mut evaluate,
        &mut keeps,
    )
    .eval_limit(limit)
    .deadline(&mut now, wall)
    .run(args.input.clone(), args.max_evals)
    .map_err(|error| ToolError::new(ErrorCategory::SandboxInfrastructure, error.message(), root))?;
    Ok(json!({
        "value": outcome.shrink.value,
        "report": serde_json::to_value(&outcome.shrink.report).unwrap_or(Value::Null),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigError;
    use crate::schemas::{BudgetArgs, ShrinkTargetArg};
    use counterspell_engine::Oracle;
    use counterspell_grimoire::runner::PackRunner;

    /// The repository root, two levels above this crate.
    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root exists")
    }

    /// A complete count-pairs pack under a fresh directory: the checked-in
    /// public tree plus the `reference/`, `incorrect/` and `known_ghosts.toml`
    /// entries the private checkout would contribute. CI has no private
    /// checkout, so the tests build their own instead of depending on a
    /// sibling directory that may not exist.
    fn pack_root() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("create private fixture directory");
        let pack = root.path().join("packs/count-pairs");
        copy_dir(&repo_root().join("packs/count-pairs"), &pack);
        fs::create_dir_all(pack.join("reference")).expect("create reference dir");
        fs::create_dir_all(pack.join("incorrect")).expect("create incorrect dir");
        fs::write(pack.join("reference/solution.py"), REFERENCE).expect("write reference");
        fs::write(pack.join("incorrect/same-index.py"), PROGRAM).expect("write same-index");
        fs::write(pack.join("incorrect/empty-min.py"), PROGRAM).expect("write empty-min");
        fs::write(pack.join("known_ghosts.toml"), KNOWN_GHOSTS).expect("write ghosts");
        fs::write(pack.join("generator.toml"), "seed = 41\nhidden_count = 5\n")
            .expect("write generator settings");
        root
    }

    /// Recursively copies a directory tree.
    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).expect("create pack dir");
        for entry in fs::read_dir(from).expect("read public pack") {
            let entry = entry.expect("readable entry");
            let target = to.join(entry.file_name());
            if entry.file_type().expect("file type").is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), &target).expect("copy pack file");
            }
        }
    }

    /// A stand-in program for `reference/` and `incorrect/` files. The scripted
    /// runner never executes it; the grimoire only checks the file exists.
    const PROGRAM: &str = "def count_pairs(values, target):\n    return 0\n";

    /// A minimal correct program for `reference/solution.py`.
    const REFERENCE: &str = "def count_pairs(values, target):\n    count = 0\n    for i in range(len(values)):\n        for j in range(i + 1, len(values)):\n            if values[i] + values[j] == target:\n                count += 1\n    return count\n";

    /// The ghosts the tests rely on, in the same shape the private pack ships.
    const KNOWN_GHOSTS: &str = r#"[[ghost]]
id = "same-index"
program = "incorrect/same-index.py"
kind = "wrong"
input = [[0], 0]
got = 1
explanation = "The inner loop pairs a value with itself."

[[ghost]]
id = "empty-min"
program = "incorrect/empty-min.py"
kind = "crash"
input = [[], 0]
exception_class = "ValueError"
explanation = "min() on an empty list raises."
"#;

    /// What the fake says the named program does on every input.
    #[derive(Clone, Copy)]
    enum ProgramBehaviour {
        /// Counts a value paired with itself, which is the authored same-index bug.
        CountsSelf,
        /// Gives the reference answer, so no failure exists.
        Correct,
        /// Raises an exception.
        Crashes,
        /// Runs past its time limit.
        TimesOut,
        /// Prints something the driver cannot read.
        Garbles,
    }

    /// A fake runner that computes count-pairs answers in Rust. It never runs Python.
    ///
    /// It knows exactly one program. Any other program name is an infrastructure fault.
    struct ScriptedRunner {
        program: &'static str,
        behaviour: ProgramBehaviour,
        reference_crashes: bool,
    }

    impl ScriptedRunner {
        fn new(program: &'static str, behaviour: ProgramBehaviour) -> Self {
            Self {
                program,
                behaviour,
                reference_crashes: false,
            }
        }

        fn program_result(&self, input: &Value) -> ProgramResult {
            match self.behaviour {
                ProgramBehaviour::CountsSelf => answer(count_pairs(input, true)),
                ProgramBehaviour::Correct => answer(count_pairs(input, false)),
                ProgramBehaviour::Crashes => ProgramResult::Crashed {
                    exception_class: "ValueError".to_string(),
                    message: "min() of an empty list".to_string(),
                },
                ProgramBehaviour::TimesOut => ProgramResult::TimedOut,
                ProgramBehaviour::Garbles => ProgramResult::Garbled {
                    snippet: "not a value".to_string(),
                },
            }
        }
    }

    /// Wraps a computed count as a returned result, or a garbled one when the input is unreadable.
    fn answer(count: Option<i64>) -> ProgramResult {
        match count {
            Some(value) => ProgramResult::Returned(json!(value)),
            None => ProgramResult::Garbled {
                snippet: "unreadable input".to_string(),
            },
        }
    }

    fn count_pairs(input: &Value, include_same: bool) -> Option<i64> {
        let values = input.get(0)?.as_array()?;
        let target = input.get(1)?.as_i64()?;
        let mut count = 0;
        for (i, first) in values.iter().enumerate() {
            for (j, second) in values.iter().enumerate() {
                let pairs = if include_same { i <= j } else { i < j };
                if pairs && first.as_i64()? + second.as_i64()? == target {
                    count += 1;
                }
            }
        }
        Some(count)
    }

    impl PackRunner for ScriptedRunner {
        fn run(
            &mut self,
            target: Target<'_>,
            input: &Value,
            _limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            match target {
                Target::Reference if self.reference_crashes => Ok(ProgramResult::Crashed {
                    exception_class: "KeyError".to_string(),
                    message: "reference broke".to_string(),
                }),
                Target::Reference => Ok(answer(count_pairs(input, false))),
                Target::Program(name) if name == self.program => Ok(self.program_result(input)),
                Target::Program(_) => Err(RunnerError::Infrastructure(
                    "the fake runner has no such program".to_string(),
                )),
            }
        }

        fn bench(
            &mut self,
            _sized_inputs: &[(u64, Value)],
            _limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            Ok(Vec::new())
        }
    }

    /// A stand-in pinned digest. Tests never read the real language catalogue.
    const FAKE_DIGEST: &str = "sha256:test-digest";

    fn fake_slot() -> RunnerSlot {
        RunnerSlot::injected(Box::new(ScriptedRunner::new(
            "incorrect/same-index.py",
            ProgramBehaviour::CountsSelf,
        )))
    }

    fn slot_for(program: &'static str, behaviour: ProgramBehaviour) -> RunnerSlot {
        RunnerSlot::injected(Box::new(ScriptedRunner::new(program, behaviour)))
    }

    fn hunt_args(program: &str) -> FindGhostArgs {
        FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: program.to_string(),
            seed_override: Some(1),
            budget: None,
        }
    }

    /// Decodes a found or not-found hunt result back into the engine type.
    fn decode(output: Value) -> counterspell_engine::HuntReport {
        serde_json::from_value(output).expect("the hunt result decodes")
    }

    fn validate_args(path: &str) -> ValidatePackArgs {
        ValidatePackArgs {
            pack_path: path.to_string(),
            deny_warnings: false,
        }
    }

    #[test]
    fn validate_pack_matches_the_grimoire_report_for_the_same_pack() {
        let root = pack_root();
        let output =
            validate_pack(root.path(), &validate_args("packs/count-pairs")).expect("tool result");
        // The tool validates the merged pair when a private checkout is
        // available, so the direct baseline must mirror that.
        let pack_dir = root.path().join("packs/count-pairs");
        let direct = match private_pack_dir(&pack_dir) {
            Some(private_dir) => {
                validate_static_pair(&pack_dir, &private_dir).expect("pair report")
            }
            None => validate_static(&pack_dir).expect("direct report"),
        };
        assert_eq!(output["report"], direct.to_json());
    }

    #[test]
    fn validate_pack_refuses_a_path_outside_the_workspace_without_reading_it() {
        let error = validate_pack(&repo_root(), &validate_args("../outside")).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn explain_verdict_uses_the_authored_known_ghost_for_a_matching_wrong_answer() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = ExplainVerdictArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!([[0], 0]),
            expected: Some(json!(0)),
            actual: Some(json!(1)),
            kind: "wrong".to_string(),
            exception_class: None,
        };
        let output = explain_verdict(pack_root().path(), &args).expect("tool result");
        assert_eq!(output["authored_mistake"]["known_ghost_id"], "same-index");
    }

    #[test]
    fn explain_verdict_lists_the_wrong_value_as_a_difference() {
        let args = ExplainVerdictArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!([[0], 0]),
            expected: Some(json!(0)),
            actual: Some(json!(1)),
            kind: "wrong".to_string(),
            exception_class: None,
        };
        let output = explain_verdict(pack_root().path(), &args).expect("tool result");
        let differences = output["differences"].as_array().expect("array");
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0]["expected"], json!(0));
        assert_eq!(differences[0]["got"], json!(1));
    }

    #[test]
    fn explain_verdict_refuses_an_unknown_kind() {
        let args = ExplainVerdictArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!([[0], 0]),
            expected: None,
            actual: None,
            kind: "maybe".to_string(),
            exception_class: None,
        };
        let error = explain_verdict(pack_root().path(), &args).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn fit_complexity_refuses_more_samples_than_the_engine_allows() {
        let samples = (0..=MAX_FIT_POINTS)
            .map(|i| crate::schemas::SampleArg {
                n: 2 + i as u64,
                median_ns: 10 + i as u64,
            })
            .collect();
        let args = FitComplexityArgs {
            samples,
            reference_complexity: "n".to_string(),
        };
        let error = fit_complexity(&repo_root(), &args).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn fit_complexity_refuses_an_unknown_reference_class() {
        let args = FitComplexityArgs {
            samples: Vec::new(),
            reference_complexity: "n!".to_string(),
        };
        let error = fit_complexity(&repo_root(), &args).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn run_reference_without_a_runner_fails_closed() {
        let args = RunReferenceArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!([[1, 2, 3, 2], 4]),
        };
        let mut slot = RunnerSlot::unconfigured();
        let error = run_reference(pack_root().path(), &mut slot, &args).expect_err("fails closed");
        assert_eq!(error.category, ErrorCategory::SandboxNotConfigured);
    }

    #[test]
    fn run_reference_returns_the_sample_answer_from_the_runner() {
        let args = RunReferenceArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!([[1, 2, 3, 2], 4]),
        };
        let output =
            run_reference(pack_root().path(), &mut fake_slot(), &args).expect("tool result");
        assert_eq!(output, json!({ "outcome": "returned", "value": 2 }));
    }

    #[test]
    fn run_reference_refuses_an_input_outside_the_schema() {
        let args = RunReferenceArgs {
            pack_path: "packs/count-pairs".to_string(),
            input: json!(["not a list", 4]),
        };
        let error =
            run_reference(pack_root().path(), &mut fake_slot(), &args).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn a_runner_fault_is_an_infrastructure_error_never_a_wrong_answer() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/empty-min.py".to_string(),
            seed_override: None,
            budget: None,
        };
        let error = find_ghost_input(pack_root().path(), &mut fake_slot(), FAKE_DIGEST, &args)
            .expect_err("aborted");
        assert_eq!(error.category, ErrorCategory::SandboxInfrastructure);
    }

    #[test]
    fn find_ghost_input_finds_a_failing_input_for_the_same_index_bug() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/same-index.py".to_string(),
            seed_override: Some(1),
            budget: Some(BudgetArgs::default()),
        };
        let output = find_ghost_input(pack_root().path(), &mut fake_slot(), FAKE_DIGEST, &args)
            .expect("tool result");
        assert!(output["outcome"].get("Found").is_some());
    }

    #[test]
    fn find_ghost_input_without_a_runner_fails_closed() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/same-index.py".to_string(),
            seed_override: None,
            budget: None,
        };
        let error = find_ghost_input(
            pack_root().path(),
            &mut RunnerSlot::unconfigured(),
            FAKE_DIGEST,
            &args,
        )
        .expect_err("fails closed");
        assert_eq!(error.category, ErrorCategory::SandboxNotConfigured);
    }

    #[test]
    fn find_ghost_input_refuses_a_program_outside_incorrect() {
        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "reference/solution.py".to_string(),
            seed_override: None,
            budget: None,
        };
        let error = find_ghost_input(pack_root().path(), &mut fake_slot(), FAKE_DIGEST, &args)
            .expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn shrink_input_keeps_a_wrong_answer_and_returns_a_failing_input() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = ShrinkInputArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/same-index.py".to_string(),
            input: json!([[0, 0, 0], 0]),
            target: ShrinkTargetArg {
                kind: "wrong".to_string(),
            },
            max_evals: 300,
            wall_ms: 3000,
        };
        let output =
            shrink_input(pack_root().path(), &mut fake_slot(), &args).expect("tool result");
        let shrunk = output["value"].clone();
        let mut fake = ScriptedRunner::new("incorrect/same-index.py", ProgramBehaviour::CountsSelf);
        let mut oracle = ProgramOracle::new(&mut fake, "incorrect/same-index.py");
        let observed = oracle
            .evaluate(&shrunk, Duration::from_secs(1))
            .expect("fake answers");
        assert!(matches!(
            classify(&observed, &counterspell_engine::Comparator::Exact),
            Classification::Fail(_)
        ));
    }

    #[test]
    fn shrink_input_refuses_a_timeout_target() {
        let args = ShrinkInputArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/same-index.py".to_string(),
            input: json!([[0], 0]),
            target: ShrinkTargetArg {
                kind: "timeout".to_string(),
            },
            max_evals: 10,
            wall_ms: 100,
        };
        let error = shrink_input(pack_root().path(), &mut fake_slot(), &args).expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn a_missing_sandbox_is_reported_as_not_configured_with_no_path_leak() {
        let error = config_error(&ConfigError::NotConfigured, &repo_root());
        assert_eq!(error.category, ErrorCategory::SandboxNotConfigured);
        assert!(!error.message.contains(&repo_root().display().to_string()));
    }

    #[test]
    fn a_crashing_program_gives_a_crash_ghost() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let mut slot = slot_for("incorrect/empty-min.py", ProgramBehaviour::Crashes);
        let output = find_ghost_input(
            pack_root().path(),
            &mut slot,
            FAKE_DIGEST,
            &hunt_args("incorrect/empty-min.py"),
        )
        .expect("tool result");
        match decode(output).outcome {
            counterspell_engine::HuntOutcome::Found(ghost) => {
                assert_eq!(
                    ghost.predicate.kind,
                    counterspell_engine::FailureKind::RuntimeError
                );
            }
            other => panic!("expected a crash ghost, got {other:?}"),
        }
    }

    #[test]
    fn a_timeout_is_not_a_default_ghost_because_the_engine_hunts_timeouts_only_when_targeted() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let mut slot = slot_for("incorrect/empty-min.py", ProgramBehaviour::TimesOut);
        let output = find_ghost_input(
            pack_root().path(),
            &mut slot,
            FAKE_DIGEST,
            &hunt_args("incorrect/empty-min.py"),
        )
        .expect("tool result");
        assert!(matches!(
            decode(output).outcome,
            counterspell_engine::HuntOutcome::NotFound(_)
        ));
    }

    #[test]
    fn a_garbled_answer_is_reported_as_a_wrong_answer() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let mut slot = slot_for("incorrect/empty-min.py", ProgramBehaviour::Garbles);
        let output = find_ghost_input(
            pack_root().path(),
            &mut slot,
            FAKE_DIGEST,
            &hunt_args("incorrect/empty-min.py"),
        )
        .expect("tool result");
        match decode(output).outcome {
            counterspell_engine::HuntOutcome::Found(ghost) => {
                assert_eq!(
                    ghost.predicate.kind,
                    counterspell_engine::FailureKind::WrongAnswer
                );
            }
            other => panic!("expected a wrong-answer ghost, got {other:?}"),
        }
    }

    #[test]
    fn a_correct_program_gives_a_not_found_result_within_budget() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/empty-min.py".to_string(),
            seed_override: Some(1),
            budget: Some(BudgetArgs {
                max_candidates: Some(40),
                search_ms: Some(1000),
                max_shrink_evals: Some(20),
                shrink_ms: Some(500),
                total_ms: Some(4000),
            }),
        };
        let mut slot = slot_for("incorrect/empty-min.py", ProgramBehaviour::Correct);
        let output = find_ghost_input(pack_root().path(), &mut slot, FAKE_DIGEST, &args)
            .expect("tool result");
        assert!(matches!(
            decode(output).outcome,
            counterspell_engine::HuntOutcome::NotFound(_)
        ));
    }

    #[test]
    fn a_reference_that_always_fails_aborts_the_hunt_as_engine_aborted() {
        // These cases read the real count-pairs private parts; without a
        // counterspell-packs-private checkout there is nothing to run them
        // against (the CI workspace has none).
        if private_pack_dir(&repo_root().join("packs/count-pairs")).is_none() {
            eprintln!("skipping: no counterspell-packs-private checkout is available");
            return;
        }

        let mut runner =
            ScriptedRunner::new("incorrect/same-index.py", ProgramBehaviour::CountsSelf);
        runner.reference_crashes = true;
        let mut slot = RunnerSlot::injected(Box::new(runner));
        let error = find_ghost_input(
            pack_root().path(),
            &mut slot,
            FAKE_DIGEST,
            &hunt_args("incorrect/same-index.py"),
        )
        .expect_err("aborted");
        assert_eq!(error.category, ErrorCategory::EngineAborted);
    }

    #[test]
    fn a_budget_over_the_eight_second_cap_is_refused() {
        let args = FindGhostArgs {
            pack_path: "packs/count-pairs".to_string(),
            program: "incorrect/same-index.py".to_string(),
            seed_override: None,
            budget: Some(BudgetArgs {
                total_ms: Some(9000),
                ..BudgetArgs::default()
            }),
        };
        let error = find_ghost_input(pack_root().path(), &mut fake_slot(), FAKE_DIGEST, &args)
            .expect_err("refused");
        assert_eq!(error.category, ErrorCategory::InvalidRequest);
    }

    #[test]
    fn fit_complexity_returns_the_engine_fit_for_the_same_samples() {
        let sizes = [256_u64, 512, 1024];
        let args = FitComplexityArgs {
            samples: sizes
                .iter()
                .map(|&n| crate::schemas::SampleArg {
                    n,
                    median_ns: 10 * n,
                })
                .collect(),
            reference_complexity: "n".to_string(),
        };
        let output = fit_complexity(&repo_root(), &args).expect("tool result");
        let points: Vec<Point> = sizes
            .iter()
            .map(|&n| Point {
                n,
                median_ns: 10 * n,
                spread: None,
            })
            .collect();
        let direct =
            complexity::fit(&points, Mode::Full, &FitConfig::default()).expect("direct fit");
        assert_eq!(
            output["report"],
            serde_json::to_value(&direct).expect("encodes")
        );
    }

    // ---- the job a factory slot receives (counterspell-mcp-power T05, T10) ----

    use crate::runner::{PackJob, RunnerFactory};
    use std::sync::{Arc, Mutex};

    /// A throwaway workspace holding one complete public pack named `job-demo`. No private tree
    /// carries that name, so the public files are all the tool can see.
    struct Workspace(PathBuf);

    impl Workspace {
        fn new() -> Self {
            // A counter plus the clock keeps parallel tests apart without naming the process.
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos());
            // nosemgrep: rust.lang.security.temp-dir.temp-dir
            let root = std::env::temp_dir().join(format!(
                "counterspell-mcp-job-{nanos}-{}",
                NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            ));
            let pack = root.join("packs/job-demo");
            std::fs::create_dir_all(pack.join("reference")).expect("reference dir");
            std::fs::create_dir_all(pack.join("incorrect")).expect("incorrect dir");
            std::fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("marker");
            std::fs::write(pack.join("problem.md"), JOB_PROBLEM).expect("problem");
            std::fs::write(pack.join("meta.toml"), JOB_META).expect("meta");
            std::fs::write(pack.join("hints.toml"), JOB_HINTS).expect("hints");
            std::fs::write(pack.join("reference/solution.py"), JOB_REFERENCE).expect("reference");
            std::fs::write(pack.join("incorrect/same-index.py"), JOB_PROGRAM).expect("program");
            std::fs::write(pack.join("known_ghosts.toml"), JOB_GHOSTS).expect("ghosts");
            std::fs::write(
                pack.join("generator.toml"),
                "seed = 1729
hidden_count = 12
",
            )
            .expect("generator");
            Self(root)
        }
    }

    impl Drop for Workspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const JOB_REFERENCE: &str = "def count_pairs(values, target):\n    return 2\n";
    const JOB_PROGRAM: &str = "def count_pairs(values, target):\n    return 3\n";
    const JOB_GHOSTS: &str = "[[ghost]]\nid = \"same-index\"\nprogram = \"incorrect/same-index.py\"\nkind = \"wrong\"\ninput = [[0], 0]\ngot = 3\nconcepts = [\"pairs\"]\nexplanation = \"A value cannot pair with itself, so the single item must not be counted.\"\n";
    const JOB_PROBLEM: &str = "# Count the Pairs\n\n## Task\nCount.\n\n## Input\nA.\n\n## Output\nB.\n\n## Examples\nC.\n\n## Constraints\nD.\n";
    const JOB_HINTS: &str = "[[hint]]\nrung = 1\ntext = \"One.\"\n\n[[hint]]\nrung = 2\ntext = \"Two.\"\n\n[[hint]]\nrung = 3\ntext = \"Three.\"\n\n[[hint]]\nrung = 4\ntext = \"Four.\"\n";
    const JOB_META: &str = "slug = \"job-demo\"\ntitle = \"Count the Pairs\"\nstatus = \"draft\"\ndifficulty = \"easy\"\ntrack = \"arrays\"\nconcepts = [\"arrays\", \"pairs\"]\norigin = \"original\"\nauthor = \"Brighton\"\nlicence = \"MIT\"\npack_version = 1\nfunction = \"count_pairs\"\nreturns = \"int\"\ncomparator = \"exact\"\n\n[[params]]\nname = \"values\"\ntype = \"list[int]\"\n\n[[params]]\nname = \"target\"\ntype = \"int\"\n\n[limits]\ntime_ms = 2000\nmem_mb = 256\n\n[input_schema]\ntype = \"tuple\"\nitems = [\n  { type = \"int_array\", min_len = 0, max_len = 30, min = -50, max = 50 },\n  { type = \"int\", min = -100, max = 100 },\n]\n\n[[samples]]\ninput = [[1, 2, 3, 2], 4]\nexpected = 2\n\n[[samples]]\ninput = [[5, 5], 10]\nexpected = 1\n";

    /// Records the job it was given and hands out the scripted fake.
    struct JobRecorder(Arc<Mutex<Vec<PackJob>>>);

    impl RunnerFactory for JobRecorder {
        fn build(
            &self,
            job: &PackJob,
        ) -> Result<Box<dyn counterspell_grimoire::runner::PackRunner + Send>, RunnerError>
        {
            self.0.lock().expect("lock").push(job.clone());
            Ok(Box::new(ScriptedRunner::new(
                "incorrect/same-index.py",
                ProgramBehaviour::CountsSelf,
            )))
        }
    }

    fn recording_slot() -> (RunnerSlot, Arc<Mutex<Vec<PackJob>>>) {
        let jobs = Arc::new(Mutex::new(Vec::new()));
        (
            RunnerSlot::with_factory(Box::new(JobRecorder(jobs.clone()))),
            jobs,
        )
    }

    /// Breaks if the sandbox side gets anything but the reference and the wire-format signature.
    #[test]
    fn run_reference_gives_the_factory_only_the_reference_and_the_wire_signature() {
        let workspace = Workspace::new();
        let (mut slot, jobs) = recording_slot();
        let args = RunReferenceArgs {
            pack_path: "packs/job-demo".to_string(),
            input: json!([[1, 2, 3], 4]),
        };

        run_reference(&workspace.0, &mut slot, &args).expect("the fake answers");

        let jobs = jobs.lock().expect("lock");
        assert_eq!(jobs.len(), 1);
        assert_eq!(
            jobs[0]
                .sources
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["reference"]
        );
        assert_eq!(jobs[0].sources["reference"], JOB_REFERENCE);
        assert_eq!(
            jobs[0].signature,
            json!({
                "protocol_version": 1,
                "function": "count_pairs",
                "parameters": [
                    {"name": "values", "type": {"kind": "list", "items": {"kind": "int"}}},
                    {"name": "target", "type": {"kind": "int"}}
                ],
                "returns": {"kind": "int"}
            })
        );
    }

    /// Breaks if a hunt hands the sandbox side files other than the reference and the one program
    /// under test (hints, the statement and sibling programs stay out).
    #[test]
    fn find_ghost_input_adds_only_the_named_program() {
        let workspace = Workspace::new();
        let (mut slot, jobs) = recording_slot();
        let mut args = hunt_args("incorrect/same-index.py");
        args.pack_path = "packs/job-demo".to_string();

        find_ghost_input(&workspace.0, &mut slot, FAKE_DIGEST, &args).expect("a hunt result");

        let jobs = jobs.lock().expect("lock");
        let keys: Vec<&str> = jobs[0].sources.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["incorrect/same-index.py", "reference"]);
        assert_eq!(jobs[0].sources["incorrect/same-index.py"], JOB_PROGRAM);
    }

    /// Breaks if shrinking probes a program the sandbox side was never given.
    #[test]
    fn shrink_input_gets_the_same_job_as_a_hunt() {
        let workspace = Workspace::new();
        let (mut slot, jobs) = recording_slot();
        let args = ShrinkInputArgs {
            pack_path: "packs/job-demo".to_string(),
            program: "incorrect/same-index.py".to_string(),
            input: json!([[0, 0, 0], 0]),
            target: ShrinkTargetArg {
                kind: "wrong".to_string(),
            },
            max_evals: 50,
            wall_ms: 2000,
        };

        shrink_input(&workspace.0, &mut slot, &args).expect("a shrink result");

        let jobs = jobs.lock().expect("lock");
        let keys: Vec<&str> = jobs[0].sources.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["incorrect/same-index.py", "reference"]);
    }

    /// Wraps the scripted fake and flips a flag when the runner is dropped.
    struct DropProbe {
        inner: ScriptedRunner,
        dropped: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.dropped
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl counterspell_grimoire::runner::PackRunner for DropProbe {
        fn run(
            &mut self,
            target: Target<'_>,
            input: &Value,
            limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            self.inner.run(target, input, limit)
        }

        fn bench(
            &mut self,
            sized_inputs: &[(u64, Value)],
            limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            self.inner.bench(sized_inputs, limit)
        }
    }

    struct ProbeFactory(Arc<std::sync::atomic::AtomicUsize>);

    impl RunnerFactory for ProbeFactory {
        fn build(
            &self,
            _job: &PackJob,
        ) -> Result<Box<dyn counterspell_grimoire::runner::PackRunner + Send>, RunnerError>
        {
            Ok(Box::new(DropProbe {
                inner: ScriptedRunner::new("incorrect/same-index.py", ProgramBehaviour::CountsSelf),
                dropped: self.0.clone(),
            }))
        }
    }

    /// Breaks if a finished tool call leaves its runner, and so its sandbox container, open until
    /// the next call. Every code-running tool must release the runner it used.
    #[test]
    fn every_code_running_tool_releases_its_runner_when_it_finishes() {
        let workspace = Workspace::new();
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut slot = RunnerSlot::with_factory(Box::new(ProbeFactory(dropped.clone())));
        let count = || dropped.load(std::sync::atomic::Ordering::SeqCst);

        let reference = RunReferenceArgs {
            pack_path: "packs/job-demo".to_string(),
            input: json!([[1, 2, 3], 4]),
        };
        run_reference(&workspace.0, &mut slot, &reference).expect("run_reference");
        assert_eq!(count(), 1, "run_reference kept its runner");

        let mut hunt = hunt_args("incorrect/same-index.py");
        hunt.pack_path = "packs/job-demo".to_string();
        find_ghost_input(&workspace.0, &mut slot, FAKE_DIGEST, &hunt).expect("find_ghost_input");
        assert_eq!(count(), 2, "find_ghost_input kept its runner");

        let shrink = ShrinkInputArgs {
            pack_path: "packs/job-demo".to_string(),
            program: "incorrect/same-index.py".to_string(),
            input: json!([[0, 0, 0], 0]),
            target: ShrinkTargetArg {
                kind: "wrong".to_string(),
            },
            max_evals: 50,
            wall_ms: 2000,
        };
        shrink_input(&workspace.0, &mut slot, &shrink).expect("shrink_input");
        assert_eq!(count(), 3, "shrink_input kept its runner");
    }

    /// Breaks if a refused request (a bad input) still leaves a runner open.
    #[test]
    fn a_refused_tool_call_does_not_leave_a_runner_open_either() {
        let workspace = Workspace::new();
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut slot = RunnerSlot::with_factory(Box::new(ProbeFactory(dropped.clone())));
        let outside_schema = RunReferenceArgs {
            pack_path: "packs/job-demo".to_string(),
            input: json!([[1000], 4]),
        };

        let error = run_reference(&workspace.0, &mut slot, &outside_schema).expect_err("refused");

        assert_eq!(error.category, ErrorCategory::InvalidRequest);
        assert_eq!(
            dropped.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "no runner was ever built for a refused request"
        );
    }
}
