//! Canonical manifest and content hash for a problem pack (problem-catalogue C06).
//!
//! A pack has two trees, `public` and `private`. Each file gets a prefixed path such as
//! `public/a.txt`, so moving a file between trees changes the hash. `approval.toml` is left out,
//! so approving a pack does not change its content hash. The canonical bytes are one line per
//! file, `path NUL sha256-hex LF`, sorted by path, and the content hash is the SHA-256 of those bytes.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The file that records approval. It is excluded from the hash.
pub const APPROVAL_FILE: &str = "approval.toml";

/// Which tree a pack file came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tree {
    /// Files that may appear in the public repository.
    Public,
    /// Files that live only in the private pack repository.
    Private,
}

impl Tree {
    /// Returns the path prefix used in the manifest for this tree.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

/// One hashed file in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// The prefixed path, for example `public/a.txt`.
    pub path: String,
    /// Which tree the file came from.
    pub tree: Tree,
    /// The lower-case hex SHA-256 of the file bytes.
    pub sha256_hex: String,
}

/// The entries and the content hash of one pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    entries: Vec<ManifestEntry>,
    content_sha256: [u8; 32],
}

/// Why a manifest could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// Two files in the same tree have the same path.
    DuplicatePath(String),
}

/// Builds the manifest for a set of pack files.
///
/// Each file is a `(tree, relative path, bytes)` triple. The order of the input does not matter.
///
/// # Errors
///
/// Returns [`ManifestError::DuplicatePath`] when two files share a prefixed path.
pub fn build_manifest(files: &[(Tree, &str, &[u8])]) -> Result<Manifest, ManifestError> {
    let mut by_path: BTreeMap<String, ManifestEntry> = BTreeMap::new();
    for (tree, relative, bytes) in files {
        if *relative == APPROVAL_FILE {
            continue;
        }
        let path = format!("{}/{relative}", tree.prefix());
        if by_path.contains_key(&path) {
            return Err(ManifestError::DuplicatePath(path));
        }
        let entry = ManifestEntry {
            path: path.clone(),
            tree: *tree,
            sha256_hex: hex(&Sha256::digest(bytes)),
        };
        by_path.insert(path, entry);
    }

    let mut canonical: Vec<u8> = Vec::new();
    for entry in by_path.values() {
        canonical.extend_from_slice(entry.path.as_bytes());
        canonical.push(0);
        canonical.extend_from_slice(entry.sha256_hex.as_bytes());
        canonical.push(b'\n');
    }
    let content_sha256: [u8; 32] = Sha256::digest(&canonical).into();

    Ok(Manifest {
        entries: by_path.into_values().collect(),
        content_sha256,
    })
}

impl Manifest {
    /// Returns every entry, sorted by prefixed path.
    #[must_use]
    pub fn entries(&self) -> &[ManifestEntry] {
        &self.entries
    }

    /// Returns the 32-byte content hash.
    #[must_use]
    pub const fn content_sha256(&self) -> &[u8; 32] {
        &self.content_sha256
    }

    /// Returns the content hash as lower-case hex.
    #[must_use]
    pub fn content_sha256_hex(&self) -> String {
        hex(&self.content_sha256)
    }

    /// Returns the public entries only, as `public/<path>` mapped to the file's hex hash.
    /// Private paths never appear here (CAT-PT9).
    #[must_use]
    pub fn public_map(&self) -> BTreeMap<String, String> {
        self.entries
            .iter()
            .filter(|e| e.tree == Tree::Public)
            .map(|e| (e.path.clone(), e.sha256_hex.clone()))
            .collect()
    }
}

/// Encodes bytes as lower-case hex.
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            // Writing to a String cannot fail.
            let _ = write!(out, "{byte:02x}");
            out
        })
}
