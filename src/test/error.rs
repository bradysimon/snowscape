use std::path::PathBuf;

/// Errors that can occur when running tests.
#[derive(Debug)]
pub enum Error {
    /// The tests directory was not found.
    TestsDirectoryNotFound(PathBuf),
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// One or more tests failed.
    TestsFailed(Vec<(String, String)>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TestsDirectoryNotFound(path) => {
                write!(f, "Tests directory not found: {}", path.display())
            }
            Error::IoError(e) => write!(f, "I/O error: {}", e),
            Error::TestsFailed(failures) => {
                writeln!(f, "{} test(s) failed:", failures.len())?;
                for (name, reason) in failures {
                    writeln!(f, "  - {}: {}", name, reason)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}
