//! Message types for test-related UI and state management.

use std::path::PathBuf;

use iced::window;

use crate::test;

use super::Interaction;

/// Messages for test recording and management.
#[derive(Debug, Clone)]
pub enum Message {
    /// Change the test window width configuration.
    ChangeWidth(String),
    /// Change the test window height configuration.
    ChangeHeight(String),
    /// Change the test name for the recording.
    ChangeTestName(String),
    /// Toggle whether to capture a snapshot at the end of the test.
    ToggleSnapshot(bool),
    /// Start recording a test for the currently selected preview.
    StartRecording,
    /// A test window was opened with the given ID.
    WindowOpened(window::Id),
    /// Record an interaction during test recording.
    RecordInteraction(Interaction),
    /// Stop recording and save the test.
    StopRecording,
    /// A screenshot was captured for the test.
    ScreenshotCaptured(window::Screenshot),
    /// Change the text in the expectation input field.
    ChangeExpectText(String),
    /// Add a text expectation to the current recording.
    AddTextExpectation,
    /// Removes the current test session after saving.
    RemoveSession,
    /// Refresh the list of discovered tests for the current preview.
    RefreshList(String),
    /// Run all tests for the current preview.
    RunAll,
    /// Run a single test by path.
    RunSingle(PathBuf),
    /// Delete a test by path.
    Delete(PathBuf),
    /// Test run completed with results.
    RunComplete(Vec<test::Outcome>),
}
