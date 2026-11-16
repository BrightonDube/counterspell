//! Helpers shared by the judge's model tests. Nothing here touches Docker.

use counterspell_judge::languages::LanguageCatalog;
use tempfile::TempDir;

/// Loads the shipped `languages/languages.toml` with a stand-in Python driver, so the
/// loader accepts the enabled Python entry. The directory must be kept alive by the caller.
///
/// `python_scratch_exec` flips Python's `exec_scratch` flag, to test a language whose
/// scratch directory may be executed from.
pub fn shipped_catalogue(python_scratch_exec: bool) -> (TempDir, LanguageCatalog) {
    let text = include_str!("../../../../languages/languages.toml");
    let python_only = text.split("[javascript]").next().unwrap().to_owned();
    let text = if python_scratch_exec {
        python_only.replace("exec_scratch = false", "exec_scratch = true")
    } else {
        python_only
    };
    let dir = TempDir::new().unwrap();
    let driver_dir = dir.path().join("drivers").join("python");
    std::fs::create_dir_all(&driver_dir).unwrap();
    std::fs::write(driver_dir.join("driver.py"), "print('driver')\n").unwrap();
    let catalog = LanguageCatalog::parse(&text, dir.path()).unwrap();
    (dir, catalog)
}

/// Loads the whole shipped `languages/languages.toml` (Python and JavaScript enabled, the
/// stretch languages disabled), with stand-in driver files for the two enabled languages.
/// The directory must be kept alive by the caller.
#[allow(dead_code)]
pub fn shipped_catalogue_all() -> (TempDir, LanguageCatalog) {
    let text = include_str!("../../../../languages/languages.toml");
    let dir = TempDir::new().unwrap();
    for driver in ["drivers/python/driver.py", "drivers/javascript/driver.js"] {
        let path = dir.path().join(driver);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "// stand-in\n").unwrap();
    }
    let catalog = LanguageCatalog::parse(text, dir.path()).unwrap();
    (dir, catalog)
}
