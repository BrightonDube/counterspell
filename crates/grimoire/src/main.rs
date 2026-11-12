//! The `validate` command for Grimoire packs.
//!
//! Exit codes: 0 when there are no errors (and no warnings under
//! `--deny-warnings`), 1 for validation errors, 2 for usage, I/O or
//! infrastructure failures.

use counterspell_grimoire::codes;
use counterspell_grimoire::report::{Diagnostic, Report};
use counterspell_grimoire::validate_static;
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

/// Validates the target and returns one report per pack.
fn reports_for(options: &Options) -> Result<Vec<Report>, String> {
    match &options.target {
        Target::Single(path) => Ok(vec![read_pack(path)?]),
        Target::All(root) => {
            let mut packs = pack_dirs(root, options.include_underscore)?;
            packs.sort();
            packs.iter().map(|path| read_pack(path)).collect()
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
        let diagnostic = Diagnostic::new(
            codes::SBX_07,
            "sandbox",
            None,
            "sandbox runner not configured",
        );
        println!("{}", diagnostic.to_text_line());
        return ExitCode::from(2);
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    run(&args)
}
