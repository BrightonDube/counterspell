//! The `validate` command for Grimoire packs.
//!
//! Exit codes: 0 when there are no errors (and no warnings under
//! `--deny-warnings`), 1 for validation errors, 2 for usage, I/O or
//! infrastructure failures.

use counterspell_grimoire::codes;
use counterspell_grimoire::report::{Diagnostic, Report};
use counterspell_grimoire::runner::PackRunner;
use counterspell_grimoire::{validate_sandbox, validate_static, SandboxFault};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The usage text printed on a bad command line.
const USAGE: &str = "usage: validate <pack-dir> [--json] [--deny-warnings] [--sandbox]\n       validate --all <packs-root> [--include-underscore] [--json] [--deny-warnings] [--sandbox]";

/// What the command validates.
#[derive(Debug, PartialEq, Eq)]
enum Target {
    /// One pack directory.
    Single(PathBuf),
    /// Every pack directory under a root.
    All(PathBuf),
}

/// The parsed command line.
#[derive(Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "each field is one command-line switch"
)]
struct Options {
    target: Target,
    json: bool,
    deny_warnings: bool,
    sandbox: bool,
    include_underscore: bool,
}

/// Reads the arguments after the program name. Only the standard library is used.
fn parse(args: &[String]) -> Result<Options, String> {
    if args.first().map(String::as_str) != Some("validate") {
        return Err(USAGE.to_owned());
    }
    let mut all = false;
    let mut options = Options {
        target: Target::Single(PathBuf::new()),
        json: false,
        deny_warnings: false,
        sandbox: false,
        include_underscore: false,
    };
    let mut positional = Vec::new();
    for arg in &args[1..] {
        match arg.as_str() {
            "--all" => all = true,
            "--json" => options.json = true,
            "--deny-warnings" => options.deny_warnings = true,
            "--sandbox" => options.sandbox = true,
            "--include-underscore" => options.include_underscore = true,
            flag if flag.starts_with("--") => {
                return Err(format!("unknown option {flag}\n{USAGE}"));
            }
            path => positional.push(PathBuf::from(path)),
        }
    }
    let [path] = <[PathBuf; 1]>::try_from(positional).map_err(|_| USAGE.to_owned())?;
    options.target = if all {
        Target::All(path)
    } else {
        Target::Single(path)
    };
    Ok(options)
}

/// The pack directories the command targets, in sorted order.
fn pack_paths(options: &Options) -> Result<Vec<PathBuf>, String> {
    match &options.target {
        Target::Single(path) => Ok(vec![path.clone()]),
        Target::All(root) => {
            let mut packs = pack_dirs(root, options.include_underscore)?;
            packs.sort();
            Ok(packs)
        }
    }
}

/// Validates the target and returns one report per pack.
fn reports_for(options: &Options) -> Result<Vec<Report>, String> {
    pack_paths(options)?
        .iter()
        .map(|path| read_pack(path))
        .collect()
}

/// Validates each pack with the runner. The first fault stops the stage and is returned as is.
fn sandbox_reports(
    paths: &[PathBuf],
    runner: &mut dyn PackRunner,
) -> Result<Vec<Report>, SandboxFault> {
    paths
        .iter()
        .map(|path| validate_sandbox(path, &mut *runner))
        .collect()
}

/// Prints a sandbox fault as one diagnostic, as JSON when `--json` is set.
fn print_fault(diagnostic: Diagnostic, json: bool) {
    if json {
        let mut report = Report::new("sandbox");
        report.push(diagnostic);
        println!("{}", report.to_json());
    } else {
        println!("{}", diagnostic.to_text_line());
    }
}

/// Runs the sandbox stage on each target. With no runner configured it reports SBX-07.
///
/// A runner fault is printed as SBX-06 and exits 2. It is never reported as a pack error (exit 1).
fn run_sandbox_stage(options: &Options, runner: Option<&mut dyn PackRunner>) -> ExitCode {
    let Some(runner) = runner else {
        let diagnostic = Diagnostic::new(
            codes::SBX_07,
            "sandbox",
            None,
            "sandbox runner not configured",
        );
        print_fault(diagnostic, options.json);
        return ExitCode::from(2);
    };
    let paths = match pack_paths(options) {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match sandbox_reports(&paths, runner) {
        Ok(reports) => print_reports(&reports, options),
        Err(fault) => {
            print_fault(fault.diagnostic(), options.json);
            ExitCode::from(2)
        }
    }
}

/// Validates one pack directory, turning an I/O failure into a message.
fn read_pack(path: &Path) -> Result<Report, String> {
    validate_static(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

/// The pack directories directly under `root`, skipping `_` packs unless asked.
fn pack_dirs(root: &Path, include_underscore: bool) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", root.display()))?;
        let is_underscore = entry.file_name().to_string_lossy().starts_with('_');
        if entry.path().is_dir() && (include_underscore || !is_underscore) {
            dirs.push(entry.path());
        }
    }
    Ok(dirs)
}

/// Prints the reports and returns the exit code they imply.
fn print_reports(reports: &[Report], options: &Options) -> ExitCode {
    if options.json {
        let value = match &options.target {
            Target::Single(_) => reports.first().map(Report::to_json).unwrap_or_default(),
            Target::All(_) => {
                serde_json::Value::Array(reports.iter().map(Report::to_json).collect())
            }
        };
        println!("{value}");
    } else {
        for report in reports {
            for diagnostic in report.diagnostics() {
                println!("{}", diagnostic.to_text_line());
            }
            println!(
                "{}: {} errors, {} warnings",
                report.pack(),
                report.error_count(),
                report.warning_count()
            );
        }
    }
    let failed = reports
        .iter()
        .any(|report| report.has_errors() || (options.deny_warnings && report.warning_count() > 0));
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Runs the command and returns its exit code.
fn run(args: &[String]) -> ExitCode {
    let options = match parse(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    if options.sandbox {
        return run_sandbox_stage(&options, None);
    }
    match reports_for(&options) {
        Ok(reports) => print_reports(&reports, &options),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    // CLI operands after argv[0]. This is a user-facing command, not a
    // security boundary, so the Semgrep args/args_os rules do not apply.
    // nosemgrep: rust.lang.security.args-os.args-os
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    run(&args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterspell_engine::ProgramResult;
    use counterspell_grimoire::runner::RunnerError;
    use serde_json::Value;
    use std::time::Duration;

    /// A runner whose sandbox is down: every call is an infrastructure fault.
    struct DownRunner;

    impl PackRunner for DownRunner {
        fn run(
            &mut self,
            _target: counterspell_grimoire::runner::Target<'_>,
            _input: &Value,
            _limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            Err(RunnerError::Infrastructure(
                "sandbox daemon unreachable".to_owned(),
            ))
        }

        fn bench(
            &mut self,
            _sized_inputs: &[(u64, Value)],
            _limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            Err(RunnerError::Infrastructure(
                "sandbox daemon unreachable".to_owned(),
            ))
        }
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn count_pairs_dir() -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packs/count-pairs")
            .display()
            .to_string()
    }

    #[test]
    fn runner_fault_is_sbx_06_from_the_sandbox_stage() {
        let paths = vec![PathBuf::from(count_pairs_dir())];

        let fault = match sandbox_reports(&paths, &mut DownRunner) {
            Err(fault) => fault,
            Ok(reports) => panic!("a runner fault must not produce reports: {}", reports.len()),
        };

        assert_eq!(fault.diagnostic().code, "SBX-06");
    }

    #[test]
    fn runner_fault_exits_two_not_one() {
        let dir = count_pairs_dir();
        let options =
            parse(&args(&["validate", &dir, "--sandbox"])).expect("the command line is valid");

        // A pack error would exit 1. An infrastructure fault must exit 2 (SBX-06).
        let code = run_sandbox_stage(&options, Some(&mut DownRunner));

        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn missing_runner_exits_two_with_sbx_07() {
        let dir = count_pairs_dir();
        let options =
            parse(&args(&["validate", &dir, "--sandbox"])).expect("the command line is valid");

        assert_eq!(run_sandbox_stage(&options, None), ExitCode::from(2));
    }
}
