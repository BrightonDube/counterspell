//! The `validate` command for Grimoire packs.
//!
//! Exit codes: 0 when there are no errors (and no warnings under
//! `--deny-warnings`), 1 for validation errors, 2 for usage, I/O or
//! infrastructure failures.

use counterspell_grimoire::codes;
use counterspell_grimoire::report::{Diagnostic, Report};
use counterspell_grimoire::runner::PackRunner;
use counterspell_grimoire::{
    validate_sandbox, validate_static, validate_static_public, SandboxFault,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The usage text printed on a bad command line.
const USAGE: &str = "usage: validate <pack-dir> [--json] [--deny-warnings] [--sandbox]\n       validate --all <packs-root> [--include-underscore] [--json] [--deny-warnings] [--sandbox] [--public]";

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
    /// Validate the public tree alone; private-resident stages skip.
    public_only: bool,
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
        public_only: false,
    };
    let mut positional = Vec::new();
    for arg in &args[1..] {
        match arg.as_str() {
            "--all" => all = true,
            "--json" => options.json = true,
            "--deny-warnings" => options.deny_warnings = true,
            "--sandbox" => options.sandbox = true,
            "--include-underscore" => options.include_underscore = true,
            "--public" => options.public_only = true,
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
    let paths = pack_paths(options)?;
    match &options.target {
        Target::Single(_) => paths
            .iter()
            .map(|path| read_pack(path, options.public_only))
            .collect(),
        Target::All(_) => paths
            .iter()
            .map(|path| read_pack_tree(path, options.public_only))
            .collect(),
    }
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
fn read_pack(path: &Path, public_only: bool) -> Result<Report, String> {
    let validate = if public_only {
        validate_static_public
    } else {
        validate_static
    };
    validate(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

/// Validates one pack under --all. A tree that carries private-only files is
/// an assembled tree and validates as such; a public-only tree validates as
/// public. --public forces the public checks even when private files sit in
/// the tree, so wrong-tree placements still surface as LAY-05.
fn read_pack_tree(path: &Path, public_only: bool) -> Result<Report, String> {
    if public_only || !has_private_files(path) {
        return read_pack(path, true);
    }
    read_pack(path, false)
}

/// Whether the pack directory carries any private-resident file. Presence of
/// one is what makes a single tree an assembled tree (amendment X1).
fn has_private_files(path: &Path) -> bool {
    const PRIVATE_FILES: [&str; 4] = [
        "hidden.toml",
        "generator.toml",
        "approval.toml",
        "known_ghosts.toml",
    ];
    PRIVATE_FILES.iter().any(|name| path.join(name).is_file())
        || path.join("reference").is_dir()
        || path.join("incorrect").is_dir()
}

/// The pack directories the --all scan validates: the directories directly
/// under root, plus every underscore pack and the children of each
/// underscore-prefixed container directory when --include-underscore is
/// given. A container (`packs/_test/`) holds packs and is not one itself.
fn pack_dirs(root: &Path, include_underscore: bool) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(root)
        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", root.display()))?;
        let is_underscore = entry.file_name().to_string_lossy().starts_with('_');
        if !entry.path().is_dir() {
            continue;
        }
        if !is_underscore {
            dirs.push(entry.path());
        } else if include_underscore {
            // A `_` directory holding pack files is a pack; one holding only
            // directories is a container of packs (`packs/_test/<slug>`).
            if entry.path().join("meta.toml").is_file() {
                dirs.push(entry.path());
            } else {
                for child in std::fs::read_dir(entry.path())
                    .map_err(|error| format!("cannot read {}: {error}", entry.path().display()))?
                {
                    let child = child
                        .map_err(|error| format!("cannot read {}: {error}", root.display()))?;
                    if child.path().is_dir() {
                        dirs.push(child.path());
                    }
                }
            }
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

    /// A minimal valid pack in a temp directory — the real `count-pairs` is
    /// split across the public and private pack trees (15 Nov decision), so a
    /// single tree is synthesized here. Returned paths stay valid until the
    /// guard drops.
    struct PackDir(PathBuf);

    impl Drop for PackDir {
        fn drop(&mut self) {
            if let Some(root) = self.0.parent() {
                let _ = std::fs::remove_dir_all(root);
            }
        }
    }

    fn pack_dir() -> PackDir {
        // nosemgrep: rust.lang.security.temp-dir.temp-dir
        let dir = std::env::temp_dir().join(format!(
            "grimoire-main-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let dir = dir.join("count-pairs");
        std::fs::create_dir_all(dir.join("reference")).expect("create pack");
        std::fs::write(dir.join("problem.md"), "# Count the Pairs\n\n## Task\nCount.\n\n## Input\nA.\n\n## Output\nB.\n\n## Examples\nC.\n\n## Constraints\nD.\n").expect("problem");
        std::fs::write(
            dir.join("meta.toml"),
            "slug = \"count-pairs\"\ntitle = \"Count the Pairs\"\nstatus = \"draft\"\ndifficulty = \"easy\"\ntrack = \"arrays\"\nconcepts = [\"arrays\"]\norigin = \"original\"\nauthor = \"Brighton\"\nlicence = \"MIT\"\npack_version = 1\nfunction = \"count_pairs\"\nreturns = \"int\"\ncomparator = \"exact\"\n\n[[params]]\nname = \"values\"\ntype = \"list[int]\"\n\n[[params]]\nname = \"target\"\ntype = \"int\"\n\n[limits]\ntime_ms = 2000\nmem_mb = 256\n\n[input_schema]\ntype = \"tuple\"\nitems = [\n  { type = \"int_array\", min_len = 0, max_len = 30, min = -50, max = 50 },\n  { type = \"int\", min = -100, max = 100 },\n]\n\n[[samples]]\ninput = [[1, 2, 3, 2], 4]\nexpected = 2\n\n[[samples]]\ninput = [[5, 5], 10]\nexpected = 1\n",
        )
        .expect("meta");
        std::fs::write(
            dir.join("hints.toml"),
            "[[hint]]\nrung = 1\ntext = \"One.\"\n\n[[hint]]\nrung = 2\ntext = \"Two.\"\n\n[[hint]]\nrung = 3\ntext = \"Three.\"\n\n[[hint]]\nrung = 4\ntext = \"Four.\"\n",
        )
        .expect("hints");
        std::fs::write(
            dir.join("reference/solution.py"),
            "def count_pairs(values, target):\n    return 0\n",
        )
        .expect("reference");
        std::fs::write(
            dir.join("generator.toml"),
            "seed = 1729
hidden_count = 12
",
        )
        .expect("generator");
        PackDir(dir)
    }

    /// The pack without its private-side  — the shape a public
    /// checkout has after the tree split.
    fn public_pack_dir() -> PackDir {
        let dir = pack_dir();
        std::fs::remove_dir_all(dir.0.join("reference")).expect("drop reference");
        std::fs::remove_file(dir.0.join("generator.toml")).expect("drop generator");
        dir
    }

    #[test]
    fn public_flag_is_parsed() {
        let options = parse(&args(&["validate", "packs/x", "--public"])).expect("options");
        assert!(options.public_only);
    }

    #[test]
    fn a_public_tree_needs_the_public_flag_to_validate() {
        let dir = public_pack_dir();
        assert!(
            read_pack(&dir.0, false).expect("report").has_errors(),
            "missing private reference must fail a full validation"
        );
        let report = read_pack(&dir.0, true).expect("public report");
        assert!(
            !report.has_errors(),
            "public validation skips the absent reference stage: {report:?}"
        );
    }

    #[test]
    fn a_reference_in_the_public_tree_is_lay_05() {
        // Amendment X1: the reference is private-only, so a copy under the
        // public tree is a wrong-tree file even though it parses.
        let dir = pack_dir();
        std::fs::write(
            dir.0.join("reference/solution.py"),
            "def count_pairs(values, target):
    return 0
",
        )
        .expect("reference");
        let report = read_pack(&dir.0, true).expect("public report");
        assert!(report.has_errors());
        assert!(report
            .diagnostics()
            .iter()
            .any(|d| d.code == "LAY-05" && d.file == "reference/solution.py"));
    }

    #[test]
    fn runner_fault_is_sbx_06_from_the_sandbox_stage() {
        let dir = pack_dir();
        let paths = vec![dir.0.clone()];

        let fault = match sandbox_reports(&paths, &mut DownRunner) {
            Err(fault) => fault,
            Ok(reports) => panic!("a runner fault must not produce reports: {}", reports.len()),
        };

        assert_eq!(fault.diagnostic().code, "SBX-06");
    }

    #[test]
    fn runner_fault_exits_two_not_one() {
        let dir = pack_dir();
        let options = parse(&args(&[
            "validate",
            &dir.0.display().to_string(),
            "--sandbox",
        ]))
        .expect("the command line is valid");

        // A pack error would exit 1. An infrastructure fault must exit 2 (SBX-06).
        let code = run_sandbox_stage(&options, Some(&mut DownRunner));

        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn missing_runner_exits_two_with_sbx_07() {
        let dir = pack_dir();
        let options = parse(&args(&[
            "validate",
            &dir.0.display().to_string(),
            "--sandbox",
        ]))
        .expect("the command line is valid");

        assert_eq!(run_sandbox_stage(&options, None), ExitCode::from(2));
    }
}
