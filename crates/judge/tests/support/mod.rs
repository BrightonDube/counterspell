//! Helpers shared by the judge's model tests. Nothing here touches Docker.

use counterspell_judge::languages::LanguageCatalog;
use tempfile::TempDir;

/// Loads the shipped `languages/languages.toml` with a stand-in Python driver, so the
/// loader accepts the enabled Python entry. The directory must be kept alive by the caller.
///
/// `python_scratch_exec` flips Python's `exec_scratch` flag, to test a language whose
/// scratch directory may be executed from.
#[allow(dead_code)]
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

/// A tracing capture for log-hygiene assertions (T35, R13.5): every formatted
/// event lands in a shared buffer so a test can prove user-controlled bytes never
/// reach the logs. Install once per test binary (`set_global_default` is global).
pub struct LogCapture {
    buf: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

#[allow(dead_code)]
impl LogCapture {
    /// Creates a capture not yet wired into tracing.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Installs this capture as the process-wide subscriber at TRACE. Panics if a
    /// global default was already set — keep one capture per test binary.
    pub fn install_global(&self) {
        let buf = self.buf.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || BufferWriter(buf.clone()))
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("one LogCapture::install_global per test binary");
    }

    /// Everything captured so far, lossily decoded.
    #[must_use]
    pub fn text(&self) -> String {
        let buf = self.buf.lock().expect("log buffer poisoned");
        String::from_utf8_lossy(&buf).into_owned()
    }
}

struct BufferWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for BufferWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut buf = self
            .0
            .lock()
            .map_err(|_| std::io::Error::other("log buffer poisoned"))?;
        buf.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
