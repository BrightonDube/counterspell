//! Import planner (problem-catalogue C11, CAT-I7 to CAT-I9). Pure: it decides, it never writes.
//!
//! The planner compares the incoming pack with the latest stored version. A `match` on [`Plan`] must
//! list every case, so a refusal can never be forgotten by the caller.

/// The latest stored version of a problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredVersion {
    /// The stored `pack_version`.
    pub version: i32,
    /// The stored content hash.
    pub hash: [u8; 32],
    /// Each stored file as `(prefixed path, hex sha256)`.
    pub files: Vec<(String, String)>,
}

/// The pack being imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incoming {
    /// The `pack_version` from `meta.toml`.
    pub pack_version: i32,
    /// The content hash computed from the files.
    pub hash: [u8; 32],
    /// Each file as `(prefixed path, hex sha256)`.
    pub files: Vec<(String, String)>,
}

/// What the importer will do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Write a new version.
    Create,
    /// The same content is already stored. Nothing is written.
    Unchanged,
    /// Do not write. The reason is in the refusal.
    Refuse(Refusal),
}

/// Why an import is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The content changed but the version number did not go up. Lists the files that changed.
    VersionNotBumped {
        /// Prefixed paths that were added, removed or changed.
        changed: Vec<String>,
    },
    /// The version number went down.
    VersionRegressed,
}

/// Decides what to do with `incoming`, given the latest stored version (if any).
#[must_use]
pub fn plan(latest: Option<&StoredVersion>, incoming: &Incoming) -> Plan {
    let Some(stored) = latest else {
        return Plan::Create;
    };
    if stored.hash == incoming.hash {
        return Plan::Unchanged;
    }
    if incoming.pack_version > stored.version {
        return Plan::Create;
    }
    if incoming.pack_version == stored.version {
        return Plan::Refuse(Refusal::VersionNotBumped {
            changed: changed_paths(&stored.files, &incoming.files),
        });
    }
    Plan::Refuse(Refusal::VersionRegressed)
}

/// Returns every path that is added, removed or whose hash differs, sorted.
fn changed_paths(stored: &[(String, String)], incoming: &[(String, String)]) -> Vec<String> {
    let mut changed: Vec<String> = Vec::new();
    for (path, hash) in incoming {
        if !stored.iter().any(|(p, h)| p == path && h == hash) {
            changed.push(path.clone());
        }
    }
    for (path, _) in stored {
        if !incoming.iter().any(|(p, _)| p == path) {
            changed.push(path.clone());
        }
    }
    changed.sort();
    changed.dedup();
    changed
}
