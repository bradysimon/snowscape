//! A builder-style API for configuring and launching the [`Emulator`] with custom settings.
use std::path::PathBuf;
use std::time::Duration;

use iced_test::core::Size;
use iced_test::emulator::Mode;
use iced_test::program::Program;

use super::{Emulator, Result};

/// Builder for an [`Emulator`].
pub struct Builder<P: Program> {
    program: P,
    mode: Mode,
    size: Size,
    preset: Option<String>,
    default_timeout: Duration,
    screenshot_dir: Option<PathBuf>,
}

impl<P: Program + 'static> Builder<P> {
    /// Creates a new builder with the given program and default configuration.
    pub fn new(program: P) -> Self {
        Builder {
            program,
            mode: Mode::default(),
            size: Size::new(1024.0, 768.0),
            preset: None,
            default_timeout: super::DEFAULT_TIMEOUT,
            screenshot_dir: None,
        }
    }

    /// Sets the headless [`Mode`] used to wait for tasks between instructions.
    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the viewport size of the emulator.
    #[must_use]
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    /// Selects a [`Preset`](iced::Preset) by name to boot the program with.
    #[must_use]
    pub fn preset(mut self, name: impl Into<String>) -> Self {
        self.preset = Some(name.into());
        self
    }

    /// Sets the default timeout used by [`Emulator::wait_for`] and friends.
    #[must_use]
    pub fn default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Enables automatic screenshot capture when a test error occurs.
    ///
    /// Screenshots are saved as PNG files in the given directory. The
    /// directory will be created if it does not exist.
    #[must_use]
    pub fn screenshot_on_failure(mut self, dir: impl Into<PathBuf>) -> Self {
        self.screenshot_dir = Some(dir.into());
        self
    }

    /// Boots the [`Emulator`].
    pub fn build(self) -> Result<Emulator<P>> {
        Emulator::boot(
            self.program,
            self.mode,
            self.size,
            self.preset,
            self.default_timeout,
            self.screenshot_dir,
        )
    }
}
