//! Message types for test-related UI and state management.

use std::path::PathBuf;

use iced::window;

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
    /// Capture a snapshot at the current point in the recording.
    CaptureSnapshot,
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
    RunComplete(Vec<TestResult>),
}

/// Result of running a single test.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// The name of the test.
    pub name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Error message if the test failed.
    pub error: Option<String>,
}
