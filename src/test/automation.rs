//! Automation testing for `iced` applications.
//!
//! This module provides a Playwright-style [`Emulator`] that wraps any
//! [`iced::Program`] and lets you drive it from plain `#[test]` functions:
//!
//! ```ignore
//! # use std::time::Duration;
//! # fn program() -> impl iced_test::program::Program { unimplemented!() }
//! #[test]
//! fn my_test() -> snowscape::test::automation::Result {
//!     let mut emulator = snowscape::test::Emulator::new(program())?;
//!     emulator.click("Increment")?;
//!     emulator.wait(Duration::from_millis(100))?;
//!     assert!(emulator.exists("Count: 1"));
//!     Ok(())
//! }
//! ```
//!
//! The [`Emulator`] runs the program with its real executor, so subscriptions
//! and tasks run just like the normal app. Use [`Emulator::wait`] to sleep and
//! drain background events, and [`Emulator::wait_for`] / [`Emulator::wait_for_text`]
//! to poll for a state change.

pub mod builder;
mod emulator;
mod interaction;
mod query;
mod screenshot;
mod widget_tree;

use builder::Builder;
pub use emulator::Emulator;
pub use interaction::{Id, IntoSelector};
pub use screenshot::Screenshot;
pub use widget_tree::{WidgetKind, WidgetNode};

use std::time::Duration;

use iced_test::Instruction;

/// Convenience re-exports for building [`Selector`](iced_test::Selector)s in automation tests.
pub mod select {
    pub use iced_test::Selector;
    pub use iced_test::selector::{Bounded, Candidate, Target, Text, id, is_focused};
}

/// The default timeout used by [`Emulator::wait_for`] and friends when no
/// explicit timeout is provided.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors that can arise from automation tests.
pub enum Error {
    /// An [`Instruction`] failed to execute (e.g. a selector did not match).
    InstructionFailed(Instruction),
    /// A [`Selector`] did not match anything.
    SelectorNotFound(String),
    /// The matched [`Target`] is not a textual widget.
    NotTextual(String),
    /// A [`wait_for`](Emulator::wait_for) call exceeded its timeout.
    Timeout {
        /// What we were waiting for.
        description: String,
        /// How long we waited.
        elapsed: Duration,
    },
    /// An assertion made via one of the `assert_*` helpers failed.
    AssertionFailed(String),
    /// The headless runtime disconnected unexpectedly.
    Disconnected,
    /// Failed to construct the headless renderer.
    Renderer(String),
    /// An I/O error occurred (e.g. saving a screenshot).
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InstructionFailed(instr) => {
                write!(f, "instruction failed: {instr}")
            }
            Error::SelectorNotFound(desc) => {
                write!(f, "selector not found: {desc}")
            }
            Error::NotTextual(desc) => {
                write!(f, "selected target is not textual: {desc}")
            }
            Error::Timeout {
                description,
                elapsed,
            } => write!(f, "timed out after {elapsed:?} waiting for: {description}"),
            Error::AssertionFailed(msg) => write!(f, "assertion failed: {msg}"),
            Error::Disconnected => write!(f, "emulator runtime disconnected"),
            Error::Renderer(msg) => write!(f, "failed to create headless renderer: {msg}"),
            Error::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use Display so multi-line messages (e.g. widget trees in assertion
        // failures) render with real newlines instead of escaped `\n`.
        write!(f, "{self}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

/// Convenience alias for automation test return types.
///
/// Defaults to `Result<(), Error>`, which is the common signature for
/// `#[test]` functions that use the [`Emulator`].
pub type Result<T = ()> = std::result::Result<T, Error>;
