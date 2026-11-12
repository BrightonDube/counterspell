//! Generator check through the engine (GEN-01, GEN-02).

use crate::codes;
use crate::meta::Meta;
use crate::report::{Diagnostic, Report};
use counterspell_engine::generate::{CandidateStream, GeneratorConfig};
use std::collections::BTreeSet;

/// Checks the seed and hidden count, then draws candidates from the engine's stream.
pub fn check(meta: &Meta, report: &mut Report) {
    if meta.generator.seed < 0 {
        report.push(Diagnostic::new(
            codes::GEN_01,
            "meta.toml",
            Some("/generator/seed"),
            "seed must be 0 to 2^63 - 1",
        ));
    }
    if !(5..=50).contains(&meta.generator.hidden_count) {
        report.push(Diagnostic::new(
            codes::GEN_01,
            "meta.toml",
            Some("/generator/hidden_count"),
            "hidden_count must be 5 to 50",
        ));
    }
    let Ok(seed) = u64::try_from(meta.generator.seed) else {
        return;
    };
    // An invalid schema is reported by MET-06. The engine stream cannot run on it.
    if meta.input_schema.check_definition().is_err() {
        return;
    }
    let stream = CandidateStream::new(
        meta.input_schema.clone(),
        seed,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    );
    let mut unique = BTreeSet::new();
    let broken = stream.take(200).any(|candidate| {
        meta.input_schema.contains(&candidate.value).is_err()
            || !unique.insert(candidate.value.to_string())
    });
    if broken {
        report.push(Diagnostic::new(codes::GEN_02, "meta.toml", Some("/generator"), "the first 200 generated inputs include an invalid or repeated value (engine or schema bug)"));
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

    #[test]
    fn valid_generator_has_no_codes() {
        assert_eq!(codes_for(&fixtures::valid_text()), Vec::<String>::new());
    }

    #[test]
    fn negative_seed_is_gen_01() {
        let text = fixtures::valid_text().replace("seed = 1729", "seed = -1");

        assert_eq!(codes_for(&text), ["GEN-01"]);
    }

    #[test]
    fn hidden_count_of_four_is_gen_01() {
        let text = fixtures::valid_text().replace("hidden_count = 12", "hidden_count = 4");

        assert_eq!(codes_for(&text), ["GEN-01"]);
    }

    #[test]
    fn hidden_count_of_fifty_one_is_gen_01() {
        let text = fixtures::valid_text().replace("hidden_count = 12", "hidden_count = 51");

        assert_eq!(codes_for(&text), ["GEN-01"]);
    }

    #[test]
    fn invalid_schema_definition_never_reaches_the_engine_stream() {
        // The engine's stream panics on min > max, so the generator check must stop first.
        // MET-06 reports the broken range, and this stage adds nothing.
        let text = fixtures::valid_text().replace("min = -50, max = 50 }", "min = 50, max = -50 }");

        assert_eq!(codes_for(&text), Vec::<String>::new());
    }
}
