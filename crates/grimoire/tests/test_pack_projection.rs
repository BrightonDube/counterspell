//! The engine's fixture projections must mirror the `_test` packs exactly
//! (grimoire-packs T29). The packs' TOML files are the single source of truth;
//! `crates/engine/tests/fixtures/<slug>.json` is a derived view, and this test
//! is the sync check that keeps the two honest.

use std::path::PathBuf;

/// The three engine fixture packs.
const PACKS: [&str; 3] = ["pair-count", "longest-run", "diagonal-sum"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(path: &PathBuf) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn toml_json(path: &PathBuf) -> serde_json::Value {
    let table = toml::from_str::<toml::Table>(&read(path))
        .unwrap_or_else(|error| panic!("invalid TOML in {}: {error}", path.display()));
    serde_json::to_value(&table).expect("TOML is JSON-shaped")
}

#[test]
fn engine_projection_mirrors_the_test_packs() {
    let root = repo_root();
    for slug in PACKS {
        let pack = root.join("packs/_test").join(slug);
        let projection: serde_json::Value = serde_json::from_str(&read(
            &root
                .join("crates/engine/tests/fixtures")
                .join(format!("{slug}.json")),
        ))
        .expect("the projection parses");

        let meta = toml_json(&pack.join("meta.toml"));
        assert_eq!(projection["slug"], slug, "{slug}: projection slug");
        assert_eq!(
            projection["problem_version"], meta["pack_version"],
            "{slug}: problem version"
        );
        assert_eq!(
            projection["schema"], meta["input_schema"],
            "{slug}: input schema"
        );
        assert_eq!(
            projection["comparator"], meta["comparator"],
            "{slug}: comparator"
        );
        assert_eq!(
            projection["concept_tags"], meta["concepts"],
            "{slug}: concept tags"
        );
        assert_eq!(projection["samples"], meta["samples"], "{slug}: samples");

        // The authored ghosts: engine `concept_tags` is the pack's `concepts`,
        // and the pack adds the `program` path the ghost shows.
        let ghosts = toml_json(&pack.join("known_ghosts.toml"));
        let toml_ghosts = ghosts["ghost"].as_array().expect("ghost array");
        let json_ghosts = projection["known_ghosts"]
            .as_array()
            .expect("projection ghost array");
        assert_eq!(toml_ghosts.len(), json_ghosts.len(), "{slug}: ghost count");
        for (toml_ghost, json_ghost) in toml_ghosts.iter().zip(json_ghosts) {
            assert_eq!(json_ghost["id"], toml_ghost["id"], "{slug}: ghost id");
            assert_eq!(
                json_ghost["input"], toml_ghost["input"],
                "{slug}: ghost input"
            );
            assert_eq!(json_ghost["kind"], toml_ghost["kind"], "{slug}: ghost kind");
            assert_eq!(
                json_ghost["concept_tags"], toml_ghost["concepts"],
                "{slug}: ghost concepts"
            );
            assert_eq!(
                json_ghost["explanation"], toml_ghost["explanation"],
                "{slug}: ghost explanation"
            );
            for field in ["got", "exception_class"] {
                let expected = toml_ghost
                    .get(field)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                assert_eq!(json_ghost[field], expected, "{slug}: ghost {field}");
            }
            // Every ghost's program is a real incorrect program in the pack.
            let program = toml_ghost["program"].as_str().expect("ghost program");
            assert!(
                pack.join(program).is_file(),
                "{slug}: ghost program {program} exists"
            );
            assert_eq!(
                json_ghost["program"], toml_ghost["program"],
                "{slug}: ghost program"
            );
        }

        // The golden vectors are the curated hidden cases.
        let hidden = toml_json(&pack.join("hidden.toml"));
        assert_eq!(
            projection["golden"]["vectors"], hidden["case"],
            "{slug}: golden vectors mirror the hidden cases"
        );

        // Every descriptor program is a real file; every incorrect program in
        // the pack has a descriptor.
        let descriptor_files: Vec<&str> = projection["incorrect"]
            .as_array()
            .expect("incorrect array")
            .iter()
            .map(|entry| entry["file"].as_str().expect("descriptor file"))
            .collect();
        for file in &descriptor_files {
            assert!(
                pack.join(file).is_file(),
                "{slug}: descriptor program {file} exists"
            );
        }
        let on_disk: Vec<String> = std::fs::read_dir(pack.join("incorrect"))
            .expect("incorrect dir")
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().to_str()?.to_owned();
                std::path::Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
                    .then(|| format!("incorrect/{name}"))
            })
            .collect();
        for file in &on_disk {
            assert!(
                descriptor_files.contains(&file.as_str()),
                "{slug}: {file} has an engine descriptor"
            );
        }
    }
}
