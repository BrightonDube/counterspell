//! Workspace-relative path checks for tool arguments.
//!
//! A tool may name only a pack directory under `packs/` or a file inside one. The
//! path must be relative, have no `..`, `.` or empty parts, and contain no symlink at
//! any step. The resolved path must stay inside the workspace root.

use std::fs;

use std::path::{Path, PathBuf};

/// The longest relative path a tool may name, in bytes.
pub const MAX_RELATIVE_BYTES: usize = 240;

/// Why a path was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathError {
    /// The text is not an allowed relative path.
    Invalid(&'static str),
    /// A step on the way is a symbolic link.
    Symlink,
    /// Nothing exists at that path.
    NotFound,
    /// The path resolves outside the workspace root.
    Outside,
}

/// Resolves `relative` inside `root` and returns the absolute path.
///
/// # Errors
///
/// Returns [`PathError`] for a bad path, a symlink, a missing entry or an escape.
pub fn resolve_inside(root: &Path, relative: &str) -> Result<PathBuf, PathError> {
    check_text(relative)?;
    let mut current = root.to_path_buf();
    for part in relative.split('/') {
        current.push(part);
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            return Err(PathError::NotFound);
        };
        if metadata.file_type().is_symlink() {
            return Err(PathError::Symlink);
        }
    }
    let root_canonical = fs::canonicalize(root).map_err(|_| PathError::NotFound)?;
    let resolved = fs::canonicalize(&current).map_err(|_| PathError::NotFound)?;
    if !resolved.starts_with(&root_canonical) {
        return Err(PathError::Outside);
    }
    Ok(resolved)
}

/// The private tree for `pack_dir`, when `COUNTERSPELL_PACKS_PRIVATE_ROOT`
/// (or a `counterspell-packs-private` checkout beside the repository) has this
/// slug. `reference/` and `incorrect/` live there since the tree split.
#[must_use]
pub fn private_pack_dir(pack_dir: &Path) -> Option<PathBuf> {
    let slug = pack_dir.file_name()?.to_str()?;
    std::env::var("COUNTERSPELL_PACKS_PRIVATE_ROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let sibling = pack_dir
                .parent()
                .and_then(Path::parent)
                .map(|repo| repo.join("../counterspell-packs-private"));
            sibling.filter(|path| path.is_dir())
        })
        .map(|root| root.join("packs").join(slug))
        .filter(|dir| dir.is_dir())
}

/// Checks the text of a relative path before anything touches the disk.
fn check_text(relative: &str) -> Result<(), PathError> {
    if relative.is_empty() {
        return Err(PathError::Invalid("the path is empty"));
    }
    if relative.len() > MAX_RELATIVE_BYTES {
        return Err(PathError::Invalid("the path is too long"));
    }
    if relative.contains('\\') || relative.contains('\0') {
        return Err(PathError::Invalid("the path has a forbidden character"));
    }
    if Path::new(relative).is_absolute() {
        return Err(PathError::Invalid(
            "the path must be relative to the workspace",
        ));
    }
    if relative
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(PathError::Invalid(
            "the path may not contain empty, . or .. parts",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A fresh directory under the system temp dir. Each test gets its own name.
    fn scratch(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let dir = std::env::temp_dir().join(format!("cs-mcp-{name}-{nanos}"));
        fs::create_dir_all(dir.join("packs/demo")).expect("create scratch tree");
        fs::write(dir.join("packs/demo/meta.toml"), "x = 1").expect("write file");
        dir
    }

    #[test]
    fn a_relative_pack_path_inside_the_root_resolves() {
        let root = scratch("ok");
        let resolved = resolve_inside(&root, "packs/demo").expect("inside the root");
        assert!(resolved.ends_with("packs/demo"));
    }

    #[test]
    fn traversal_and_absolute_paths_are_refused_before_disk_access() {
        let root = scratch("trav");
        assert!(matches!(
            resolve_inside(&root, "../etc"),
            Err(PathError::Invalid(_))
        ));
        assert!(matches!(
            resolve_inside(&root, "/etc/passwd"),
            Err(PathError::Invalid(_))
        ));
        assert!(matches!(
            resolve_inside(&root, "packs/./demo"),
            Err(PathError::Invalid(_))
        ));
    }

    #[test]
    fn a_symlink_step_is_refused() {
        let root = scratch("link");
        symlink("/tmp", root.join("packs/escape")).expect("create symlink");
        assert_eq!(
            resolve_inside(&root, "packs/escape/anything"),
            Err(PathError::Symlink)
        );
    }

    #[test]
    fn a_missing_path_is_not_found() {
        let root = scratch("miss");
        assert_eq!(
            resolve_inside(&root, "packs/nothing-here"),
            Err(PathError::NotFound)
        );
    }

    #[test]
    fn a_backslash_or_nul_is_refused() {
        let root = scratch("bad");
        assert!(matches!(
            resolve_inside(&root, "packs\\demo"),
            Err(PathError::Invalid(_))
        ));
        assert!(matches!(
            resolve_inside(&root, "packs/de\0mo"),
            Err(PathError::Invalid(_))
        ));
    }
}
