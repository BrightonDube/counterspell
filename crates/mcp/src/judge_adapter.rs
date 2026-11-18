//! Loads one pack for the MCP and builds the engine request for a hunt.
//!
//! The pack's metadata, samples, schema, comparator and known ghosts come from the
//! Grimoire library. The hunt identity comes from the code bytes and the pinned
//! language image. Nothing here runs code. Code runs only through the runner that
//! the tool handlers receive.

use crate::errors::{ErrorCategory, ToolError};
use crate::paths::private_pack_dir;
use counterspell_engine::{HuntIdentity, HuntRequest};
use counterspell_grimoire::ghosts::{sandbox_ghosts, SandboxGhost};
use counterspell_grimoire::meta::{self, Meta};
use counterspell_grimoire::pack::{self, Pack};
use counterspell_grimoire::report::Report;
use counterspell_judge::languages::LanguageCatalog;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::Path;

/// The language every pack's incorrect programs and reference are written in.
pub const PACK_LANGUAGE: &str = "python";

/// One pack, loaded and checked for a tool that needs its metadata.
pub struct PackContext {
    /// The problem slug from `meta.toml`.
    pub slug: String,
    /// The parsed metadata.
    pub meta: Meta,
    /// The authored known ghosts that the sandbox can use.
    pub known: Vec<SandboxGhost>,
    /// The pack's allowed files.
    pub pack: Pack,
}

/// Loads the pack at `pack_dir` (already resolved inside the workspace).
///
/// # Errors
///
/// Returns `InvalidRequest` when `meta.toml` is missing or does not parse, and
/// `Internal` when the directory cannot be read.
pub fn load_pack(pack_dir: &Path, root: &Path) -> Result<PackContext, ToolError> {
    let dir_name = pack_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut report = Report::new(dir_name);
    // `reference/` and `incorrect/` live in the private pack tree, so the merged
    // view is used whenever a private checkout is available.
    let pack = match private_pack_dir(pack_dir) {
        Some(private_dir) => pack::load_pair(pack_dir, &private_dir, &mut report),
        None => pack::load(pack_dir, &mut report),
    }
    .map_err(|_| ToolError::new(ErrorCategory::Internal, "the pack could not be read", root))?;
    let text = pack.text("meta.toml").ok_or_else(|| {
        ToolError::new(
            ErrorCategory::InvalidRequest,
            "meta.toml is missing or rejected",
            root,
        )
    })?;
    let meta = meta::check(text, dir_name, &mut report).ok_or_else(|| {
        ToolError::new(
            ErrorCategory::InvalidRequest,
            "meta.toml does not parse",
            root,
        )
    })?;
    let known = pack
        .text("known_ghosts.toml")
        .map(sandbox_ghosts)
        .unwrap_or_default();
    Ok(PackContext {
        slug: meta.slug.clone(),
        meta,
        known,
        pack,
    })
}

impl PackContext {
    /// Builds the engine request for a hunt of `program_text`, with pinned identity fields.
    #[must_use]
    pub fn hunt_request(&self, program_text: &[u8], image_digest: &str) -> HuntRequest {
        let identity = HuntIdentity {
            problem_slug: self.slug.clone(),
            code_sha256: sha256_hex(program_text),
            language: PACK_LANGUAGE.to_string(),
            problem_version: u32::try_from(self.meta.pack_version).unwrap_or(0),
            image_digest: image_digest.to_string(),
        };
        let mut request = HuntRequest::new(
            self.meta.input_schema.clone(),
            self.meta.engine_comparator(),
            identity,
        );
        request.samples = self
            .meta
            .samples
            .iter()
            .map(|sample| sample.input.clone())
            .collect();
        request.known_ghosts = self.known.iter().map(|ghost| ghost.known.clone()).collect();
        request.concept_tags.clone_from(&self.meta.concepts);
        request
    }
}

/// The pinned image digest for the pack language, read from the judge's catalogue.
///
/// # Errors
///
/// Returns `Internal` when the catalogue cannot be read or has no Python entry.
pub fn python_image_digest(languages_file: &Path, root: &Path) -> Result<String, ToolError> {
    let catalog = LanguageCatalog::load(languages_file).map_err(|_| {
        ToolError::new(
            ErrorCategory::Internal,
            "the language catalogue is unreadable",
            root,
        )
    })?;
    catalog
        .get(PACK_LANGUAGE)
        .map(|definition| definition.image().as_str().to_string())
        .ok_or_else(|| {
            ToolError::new(
                ErrorCategory::Internal,
                "the language catalogue has no python entry",
                root,
            )
        })
}

/// The lowercase hex SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            // Writing into a String cannot fail, so the result is ignored on purpose.
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_empty_input_is_the_standard_digest() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
