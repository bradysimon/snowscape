use std::path::PathBuf;

use iced::Size;

/// Configuration for a test recording session.
#[derive(Debug, Clone)]
pub struct Config {
    /// The size of the test window.
    pub window_size: Size,
    /// The directory where tests are saved.
    pub tests_dir: PathBuf,
    /// Whether to capture a final snapshot for comparison.
    pub capture_snapshot: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_size: Size::new(800.0, 600.0),
            tests_dir: PathBuf::from("./tests"),
            capture_snapshot: false,
        }
    }
}

impl Config {
    /// Creates a new test configuration with the given window size.
    pub fn with_window_size(mut self, size: Size) -> Self {
        self.window_size = size;
        self
    }

    /// Sets the directory where tests are saved.
    pub fn with_tests_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.tests_dir = dir.into();
        self
    }

    /// Enables snapshot capture for the test.
    pub fn with_snapshot(mut self) -> Self {
        self.capture_snapshot = true;
        self
    }
}
