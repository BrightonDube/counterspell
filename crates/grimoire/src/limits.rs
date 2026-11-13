//! Launch resource caps (MET-08) and benchmark declarations (MET-09).

use crate::codes;
use crate::meta::{Bench, Meta};
use crate::report::{Diagnostic, Report};
use crate::signature::LaunchType;
use serde::Deserialize;

/// The per-test limits declared in `[limits]`. Range checks come with the launch caps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The per-test time limit in milliseconds.
    pub time_ms: i64,
    /// The memory limit in megabytes.
    pub mem_mb: i64,
}

/// Checks the launch caps and the complexity and bench pairing.
pub fn check(meta: &Meta, report: &mut Report) {
    if !(100..=2000).contains(&meta.limits.time_ms) {
        report.push(bad_limit("/limits/time_ms", "time_ms must be 100 to 2000"));
    }
    if !(64..=256).contains(&meta.limits.mem_mb) {
        report.push(bad_limit("/limits/mem_mb", "mem_mb must be 64 to 256"));
    }
    check_complexity(meta, report);
}

/// The fit-model classes a pack may declare (system design §8b).
const COMPLEXITY_CLASSES: [&str; 7] = ["1", "log n", "n", "n log n", "n^2", "n^3", "2^n"];

/// A MET-08 diagnostic for a limit outside the launch caps.
fn bad_limit(pointer: &str, message: &str) -> Diagnostic {
    Diagnostic::new(codes::MET_08, "meta.toml", Some(pointer), message)
}

/// A MET-09 diagnostic for a complexity or bench problem.
fn bad_bench(pointer: &str, message: &str) -> Diagnostic {
    Diagnostic::new(codes::MET_09, "meta.toml", Some(pointer), message)
}

/// Checks that `reference_complexity` and `[bench]` come as a pair and are valid.
fn check_complexity(meta: &Meta, report: &mut Report) {
    match (&meta.reference_complexity, &meta.bench) {
        (None, None) => {}
        (Some(class), Some(bench)) => {
            if !COMPLEXITY_CLASSES.contains(&class.as_str()) {
                report.push(bad_bench(
                    "/reference_complexity",
                    "reference_complexity must be one of 1, log n, n, n log n, n^2, n^3, 2^n",
                ));
            }
            check_bench(meta, bench, report);
        }
        (Some(_), None) => report.push(bad_bench(
            "/bench",
            "reference_complexity needs a [bench] table",
        )),
        (None, Some(_)) => report.push(bad_bench(
            "/reference_complexity",
            "[bench] needs reference_complexity",
        )),
    }
}

/// Checks the bench table: a sizeable parameter and 3 to 8 strictly increasing sizes.
fn check_bench(meta: &Meta, bench: &Bench, report: &mut Report) {
    let sizable = meta.params.iter().any(|param| {
        param.name == bench.size_param
            && matches!(
                LaunchType::parse(&param.kind),
                Some(LaunchType::IntArray | LaunchType::Str | LaunchType::Matrix)
            )
    });
    if !sizable {
        report.push(bad_bench(
            "/bench/size_param",
            "size_param must name an int array, string or matrix parameter",
        ));
    }
    if !(3..=8).contains(&bench.sizes.len()) {
        report.push(bad_bench("/bench/sizes", "sizes must have 3 to 8 values"));
    }
    if !bench.sizes.windows(2).all(|pair| pair[0] < pair[1]) {
        report.push(bad_bench(
            "/bench/sizes",
            "sizes must be strictly increasing",
        ));
    }
    if bench.sizes.iter().any(|size| *size > 65_536) {
        report.push(bad_bench("/bench/sizes", "sizes must not exceed 65536"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    fn codes_for(text: &str) -> Vec<String> {
        let meta = fixtures::parse(text);
        let mut report = Report::new("count-pairs");
        check(&meta, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    /// Adds a top-level `reference_complexity` line after `comparator`.
    fn with_complexity(complexity: &str) -> String {
        fixtures::valid_text().replace(
            "comparator = \"exact\"\n",
            &format!("comparator = \"exact\"\nreference_complexity = \"{complexity}\"\n"),
        )
    }

    /// Appends a `[bench]` table to the valid text.
    fn with_bench(size_param: &str, sizes: &str) -> String {
        format!(
            "{}\n[bench]\nsize_param = \"{size_param}\"\nsizes = {sizes}\n",
            fixtures::valid_text()
        )
    }

    #[test]
    fn time_limits_at_the_edges_are_allowed() {
        let low = fixtures::valid_text().replace("time_ms = 2000", "time_ms = 100");
        let high = fixtures::valid_text();

        assert_eq!(codes_for(&low), Vec::<String>::new());
        assert_eq!(codes_for(&high), Vec::<String>::new());
    }

    #[test]
    fn time_limit_below_100_ms_is_met_08() {
        let text = fixtures::valid_text().replace("time_ms = 2000", "time_ms = 99");

        assert_eq!(codes_for(&text), ["MET-08"]);
    }

    #[test]
    fn time_limit_above_2000_ms_is_met_08() {
        let text = fixtures::valid_text().replace("time_ms = 2000", "time_ms = 2001");

        assert_eq!(codes_for(&text), ["MET-08"]);
    }

    #[test]
    fn memory_limit_below_64_mb_is_met_08() {
        let text = fixtures::valid_text().replace("mem_mb = 256", "mem_mb = 63");

        assert_eq!(codes_for(&text), ["MET-08"]);
    }

    #[test]
    fn memory_limit_above_256_mb_is_met_08() {
        let text = fixtures::valid_text().replace("mem_mb = 256", "mem_mb = 257");

        assert_eq!(codes_for(&text), ["MET-08"]);
    }

    #[test]
    fn complexity_without_bench_is_met_09() {
        assert_eq!(codes_for(&with_complexity("n")), ["MET-09"]);
    }

    #[test]
    fn bench_without_complexity_is_met_09() {
        let text = with_bench("values", "[256, 1024, 4096]");

        assert_eq!(codes_for(&text), ["MET-09"]);
    }

    #[test]
    fn complexity_outside_the_fit_model_set_is_met_09() {
        let text = with_complexity("n log^2 n");
        let text = format!("{text}\n[bench]\nsize_param = \"values\"\nsizes = [256, 1024, 4096]\n");

        assert_eq!(codes_for(&text), ["MET-09"]);
    }

    #[test]
    fn valid_complexity_with_bench_is_allowed() {
        let text = format!(
            "{}\n[bench]\nsize_param = \"values\"\nsizes = [256, 1024, 4096]\n",
            with_complexity("n log n")
        );

        assert_eq!(codes_for(&text), Vec::<String>::new());
    }

    #[test]
    fn bench_size_param_that_is_an_int_is_met_09() {
        let text = format!(
            "{}\n[bench]\nsize_param = \"target\"\nsizes = [256, 1024, 4096]\n",
            with_complexity("n")
        );

        assert_eq!(codes_for(&text), ["MET-09"]);
    }

    #[test]
    fn bench_with_two_sizes_is_met_09() {
        let text = format!(
            "{}\n[bench]\nsize_param = \"values\"\nsizes = [256, 1024]\n",
            with_complexity("n")
        );

        assert_eq!(codes_for(&text), ["MET-09"]);
    }

    #[test]
    fn bench_sizes_that_do_not_increase_are_met_09() {
        let text = format!(
            "{}\n[bench]\nsize_param = \"values\"\nsizes = [256, 128, 4096]\n",
            with_complexity("n")
        );

        assert_eq!(codes_for(&text), ["MET-09"]);
    }

    #[test]
    fn bench_size_above_65536_is_met_09() {
        let text = format!(
            "{}\n[bench]\nsize_param = \"values\"\nsizes = [256, 1024, 65537]\n",
            with_complexity("n")
        );

        assert_eq!(codes_for(&text), ["MET-09"]);
    }
}
