//! Pure catalogue tests for C07 (taxonomy), C08 (discovery), C09 (starter code) and
//! C10 (constraints). No database and no filesystem.

use counterspell_judge::catalogue::discover::{classify, Decision, DirEntry, SkipReason};
use counterspell_judge::catalogue::starter::{starter, Lang, Param, Signature, StarterError};
use counterspell_judge::catalogue::statement::constraints;
use counterspell_judge::catalogue::taxonomy::{parse_taxonomy, TaxonomyError};

// ---- C07 taxonomy -----------------------------------------------------------

#[test]
fn taxonomy_parses_tracks_and_concepts() {
    let text = "\
[[tracks]]
slug = \"arrays\"
name = \"Arrays\"

[[concepts]]
slug = \"hash-maps\"
name = \"Hash maps\"
";
    let taxonomy = parse_taxonomy(text).expect("a valid taxonomy must parse");
    assert_eq!(taxonomy.tracks.len(), 1);
    assert_eq!(taxonomy.tracks[0].slug, "arrays");
    assert_eq!(taxonomy.concepts[0].name, "Hash maps");
}

#[test]
fn taxonomy_refuses_duplicate_slug() {
    let text = "\
[[tracks]]
slug = \"arrays\"
name = \"Arrays\"

[[tracks]]
slug = \"arrays\"
name = \"Arrays again\"
";
    let error = parse_taxonomy(text).expect_err("a duplicate track slug must be refused");
    assert_eq!(
        error,
        TaxonomyError::DuplicateSlug {
            kind: "track",
            slug: "arrays".to_string()
        }
    );
}

#[test]
fn taxonomy_refuses_unknown_key() {
    let text = "\
[[concepts]]
slug = \"hash-maps\"
name = \"Hash maps\"
colour = \"green\"
";
    let error = parse_taxonomy(text).expect_err("an unknown key must be refused");
    assert!(matches!(error, TaxonomyError::Parse(_)), "got: {error:?}");
}

#[test]
fn taxonomy_refuses_bad_slug() {
    let text = "\
[[tracks]]
slug = \"Bad_Slug\"
name = \"Bad\"
";
    let error = parse_taxonomy(text).expect_err("a slug outside the kebab rule must be refused");
    assert_eq!(error, TaxonomyError::BadSlug("Bad_Slug".to_string()));
}

// ---- C08 discovery ----------------------------------------------------------

fn pack<'a>(name: &'a str, status: Option<&'a str>) -> DirEntry<'a> {
    DirEntry {
        name,
        is_dir: true,
        has_meta: status.is_some(),
        status,
    }
}

#[test]
fn discovery_skips_underscore_directories() {
    assert_eq!(
        classify(&pack("_test", Some("published"))),
        Decision::Skip(SkipReason::Underscore)
    );
    assert_eq!(
        classify(&pack("_dogfood", Some("published"))),
        Decision::Skip(SkipReason::Underscore)
    );
}

#[test]
fn discovery_skips_draft_packs() {
    assert_eq!(
        classify(&pack("count-pairs", Some("draft"))),
        Decision::Skip(SkipReason::Draft)
    );
}

#[test]
fn discovery_imports_published_packs() {
    assert_eq!(
        classify(&pack("count-pairs", Some("published"))),
        Decision::Import
    );
}

#[test]
fn discovery_skips_plain_files() {
    let readme = DirEntry {
        name: "README.md",
        is_dir: false,
        has_meta: false,
        status: None,
    };
    assert_eq!(classify(&readme), Decision::Skip(SkipReason::NotAPack));
}

#[test]
fn discovery_refuses_explicit_underscore_slug() {
    let error = counterspell_judge::catalogue::discover::explicit_slug("_x")
        .expect_err("an explicit _ slug must be refused");
    assert_eq!(
        error.to_string(),
        "slug \"_x\" starts with _ and is never imported"
    );
    assert!(counterspell_judge::catalogue::discover::explicit_slug("count-pairs").is_ok());
}

// ---- C09 starter code -------------------------------------------------------

const COUNT_PAIRS_PARAMS: [Param; 2] = [
    Param {
        name: "values",
        kind: "list[int]",
    },
    Param {
        name: "target",
        kind: "int",
    },
];

#[test]
fn starter_python_golden() {
    let code = starter(
        Lang::Python,
        &Signature {
            function_name: "count_pairs",
            params: &COUNT_PAIRS_PARAMS,
            returns: "int",
        },
    )
    .expect("a valid signature must render");
    assert_eq!(
        code,
        "def count_pairs(values, target):\n    raise NotImplementedError(\"not implemented\")\n"
    );
}

#[test]
fn starter_javascript_golden() {
    let code = starter(
        Lang::JavaScript,
        &Signature {
            function_name: "count_pairs",
            params: &COUNT_PAIRS_PARAMS,
            returns: "int",
        },
    )
    .expect("a valid signature must render");
    assert_eq!(
        code,
        "function count_pairs(values, target) {\n  throw new Error(\"not implemented\");\n}\n"
    );
}

#[test]
fn starter_java_golden() {
    let code = starter(
        Lang::Java,
        &Signature {
            function_name: "count_pairs",
            params: &COUNT_PAIRS_PARAMS,
            returns: "int",
        },
    )
    .expect("a valid signature must render");
    assert_eq!(
        code,
        "public class Solution {\n    public static long count_pairs(long[] values, long target) {\n        throw new UnsupportedOperationException(\"not implemented\");\n    }\n}\n"
    );
}

#[test]
fn starter_cpp_golden() {
    let code = starter(
        Lang::Cpp,
        &Signature {
            function_name: "count_pairs",
            params: &COUNT_PAIRS_PARAMS,
            returns: "int",
        },
    )
    .expect("a valid signature must render");
    assert_eq!(
        code,
        "long long count_pairs(std::vector<long long> values, long long target) {\n    throw std::runtime_error(\"not implemented\");\n}\n\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {\n    return to_json(count_pairs(from_json<std::vector<long long>>(args[0]), from_json<long long>(args[1])));\n}\n"
    );
}

#[test]
fn starter_refuses_parameter_with_space() {
    let error = starter(
        Lang::Python,
        &Signature {
            function_name: "count_pairs",
            params: &[Param {
                name: "bad name",
                kind: "int",
            }],
            returns: "int",
        },
    )
    .expect_err("a parameter name with a space must be refused");
    assert_eq!(error, StarterError::InvalidName("bad name".to_string()));
}

#[test]
fn starter_refuses_an_unknown_type_for_java_and_cpp() {
    let params = [Param {
        name: "mystery",
        kind: "float",
    }];
    for lang in [Lang::Java, Lang::Cpp] {
        let error = starter(
            lang,
            &Signature {
                function_name: "f",
                params: &params,
                returns: "int",
            },
        )
        .expect_err("a non-launch type cannot be mapped to a typed starter");
        assert_eq!(error, StarterError::UnknownType("float".to_string()));
    }
}

// ---- C10 constraints --------------------------------------------------------

#[test]
fn constraints_returns_bullets_in_order() {
    let statement = "# Two Sum\n\n## Constraints\n- 2 <= n <= 10^4\n- values are integers\n";
    assert_eq!(
        constraints(statement),
        vec!["2 <= n <= 10^4", "values are integers"]
    );
}

#[test]
fn constraints_empty_when_section_missing() {
    assert_eq!(
        constraints("# Two Sum\n\nNo limits here.\n"),
        Vec::<String>::new()
    );
}

#[test]
fn constraints_stops_at_next_heading() {
    let statement = "## Constraints\n- one\n- two\n## Examples\n- not a constraint\n";
    assert_eq!(constraints(statement), vec!["one", "two"]);
}
