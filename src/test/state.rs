//! Test state management encapsulating all test-related functionality.

use std::path::PathBuf;

use iced::{Task, window};

use super::{
    Config, Session,
    discovery::{TestInfo, delete_test, discover_tests},
    message::{Message, TestResult},
};

/// Encapsulates all test-related state and logic.
#[derive(Debug)]
pub struct State {
    /// Test configuration for the test window.
    pub config: Config,
    /// The width input for the test window (as string for text input).
    pub width_input: String,
    /// The height input for the test window (as string for text input).
    pub height_input: String,
    /// The test name input for naming new tests.
    pub name_input: String,
    /// The active test recording session, if any.
    pub session: Option<Session>,
    /// The ID of the test window when recording.
    pub window_id: Option<window::Id>,
    /// Discovered tests for the current preview.
    pub discovered_tests: Vec<TestInfo>,
    /// Results from the last test run.
    pub last_run_results: Option<Vec<TestResult>>,
    /// The scope of the current test run, if any.
    pub run_mode: Option<RunMode>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            config: Config::default(),
            width_input: "800".to_string(),
            height_input: "600".to_string(),
            name_input: String::new(),
            session: None,
            window_id: None,
            discovered_tests: Vec::new(),
            last_run_results: None,
            run_mode: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RunMode {
    All,
    Single,
}

/// Context needed for test operations.
pub struct UpdateContext<'a> {
    /// The name of the currently selected preview.
    pub preview_name: &'a str,
    /// The index of the currently selected preview.
    pub preview_index: usize,
    /// Callback to build a fresh app for running tests.
    pub configure: Option<crate::app::ConfigureFn>,
}

impl State {
    /// Creates a new test state with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if a test recording is currently active.
    pub fn is_recording(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.is_recording)
    }

    /// Returns true if a test can be started (name is entered).
    pub fn can_record(&self) -> bool {
        !self.name_input.trim().is_empty()
    }

    /// Returns the current test session, if any.
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// Returns true if a test run is currently active.
    pub fn is_running(&self) -> bool {
        self.run_mode.is_some()
    }

    /// Returns the test name from user input.
    fn test_name(&self) -> String {
        self.name_input.trim().to_string()
    }

    /// Updates the test state based on the given message.
    ///
    /// Returns a task that may need to be executed.
    pub fn update(&mut self, message: Message, ctx: Option<UpdateContext<'_>>) -> Task<Message> {
        match message {
            Message::ChangeWidth(width) => {
                self.width_input = width.clone();
                if let Ok(w) = width.parse::<f32>() {
                    self.config.window_size.width = w;
                }
                Task::none()
            }
            Message::ChangeHeight(height) => {
                self.height_input = height.clone();
                if let Ok(h) = height.parse::<f32>() {
                    self.config.window_size.height = h;
                }
                Task::none()
            }
            Message::ChangeTestName(name) => {
                self.name_input = name;
                Task::none()
            }
            Message::ToggleSnapshot(enabled) => {
                self.config.capture_snapshot = enabled;
                Task::none()
            }
            Message::StartRecording => {
                let Some(ctx) = ctx else {
                    return Task::none();
                };

                // Require a test name
                if !self.can_record() {
                    return Task::none();
                }

                // Create the test session with the test name
                let test_name = self.test_name();
                let session = Session::new(
                    self.config.clone(),
                    ctx.preview_index,
                    ctx.preview_name.to_string(),
                    test_name,
                );
                self.session = Some(session);

                // Clear the name input for next time
                self.name_input.clear();

                // Open the test window
                let (id, open_task) = window::open(window::Settings {
                    size: self.config.window_size,
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.window_id = Some(id);

                open_task.map(Message::WindowOpened)
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::RecordInteraction(interaction) => {
                if let Some(session) = &mut self.session {
                    session.record(interaction);
                }
                Task::none()
            }
            Message::StopRecording => {
                let Some(session) = &self.session else {
                    return Task::none();
                };

                // Save the test file
                if let Err(e) = session.save() {
                    eprintln!("Failed to save test: {}", e);
                }

                // Close the test window
                if let Some(test_window_id) = self.window_id.take() {
                    if session.config.capture_snapshot {
                        window::screenshot(test_window_id)
                            .map(Message::ScreenshotCaptured)
                            .chain(window::close(test_window_id))
                            .chain(Task::done(Message::RemoveSession))
                    } else {
                        window::close(test_window_id).chain(Task::done(Message::RemoveSession))
                    }
                } else {
                    Task::none()
                }
            }
            Message::ScreenshotCaptured(screenshot) => {
                if let Some(session) = &mut self.session {
                    let snapshot_name = session.next_snapshot_name();
                    let snapshot_path = session.preview_dir().join(&snapshot_name);

                    if let Err(e) = std::fs::create_dir_all(session.preview_dir()) {
                        eprintln!("Failed to create tests directory: {}", e);
                    } else {
                        let width = screenshot.size.width;
                        let height = screenshot.size.height;
                        let rgba_data: &[u8] = screenshot.as_ref();

                        match image::RgbaImage::from_raw(width, height, rgba_data.to_vec()) {
                            Some(img) => {
                                if let Err(e) = img.save(&snapshot_path) {
                                    eprintln!("Failed to save snapshot as PNG: {}", e);
                                }
                            }
                            None => {
                                eprintln!("Failed to create image from screenshot data");
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::ChangeExpectText(text) => {
                if let Some(session) = &mut self.session {
                    session.expect_text_input = text;
                }
                Task::none()
            }
            Message::AddTextExpectation => {
                if let Some(session) = &mut self.session {
                    let text = std::mem::take(&mut session.expect_text_input);
                    session.add_text_expectation(text);
                }
                Task::none()
            }
            Message::CaptureSnapshot => {
                if let Some(test_window_id) = self.window_id {
                    window::screenshot(test_window_id).map(Message::ScreenshotCaptured)
                } else {
                    Task::none()
                }
            }
            Message::RemoveSession => {
                // Get the preview name before clearing session for refresh
                let preview_name = self
                    .session
                    .as_ref()
                    .map(|s| s.preview_name.clone())
                    .unwrap_or_default();

                self.session = None;

                // Refresh the test list
                if !preview_name.is_empty() {
                    Task::done(Message::RefreshList(preview_name))
                } else {
                    Task::none()
                }
            }
            Message::RefreshList(preview_name) => {
                self.discovered_tests = discover_tests(&self.config.tests_dir, &preview_name);
                Task::none()
            }
            Message::RunAll => {
                if self.discovered_tests.is_empty() || self.is_running() {
                    return Task::none();
                }

                let Some(ctx) = ctx else {
                    return Task::none();
                };

                let Some(configure) = ctx.configure.clone() else {
                    return Task::none();
                };

                self.run_mode = Some(RunMode::All);
                self.last_run_results = None;

                // For now, run tests synchronously in a simple manner
                // In a full implementation, this would spawn a background task
                let tests: Vec<PathBuf> = self
                    .discovered_tests
                    .iter()
                    .map(|t| t.path.clone())
                    .collect();

                let preview_index = ctx.preview_index;

                Task::perform(
                    async move { run_tests_in_thread(configure, preview_index, tests).await },
                    Message::RunComplete,
                )
            }
            Message::RunSingle(path) => {
                if self.is_running() {
                    return Task::none();
                }

                let Some(ctx) = ctx else {
                    return Task::none();
                };

                let Some(configure) = ctx.configure.clone() else {
                    return Task::none();
                };

                self.run_mode = Some(RunMode::Single);

                let preview_index = ctx.preview_index;

                Task::perform(
                    async move { run_tests_in_thread(configure, preview_index, vec![path]).await },
                    Message::RunComplete,
                )
            }
            Message::Delete(path) => {
                if let Err(e) = delete_test(&path) {
                    eprintln!("Failed to delete test: {}", e);
                }

                // Refresh the list - we need the preview name
                let preview_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                self.discovered_tests = discover_tests(&self.config.tests_dir, &preview_name);
                Task::none()
            }
            Message::RunComplete(results) => {
                self.last_run_results = Some(match self.run_mode {
                    Some(RunMode::Single) => {
                        merge_run_results(self.last_run_results.take(), results)
                    }
                    _ => results,
                });
                self.run_mode = None;
                Task::none()
            }
        }
    }
}

fn merge_run_results(
    previous: Option<Vec<TestResult>>,
    updates: Vec<TestResult>,
) -> Vec<TestResult> {
    let Some(mut existing) = previous else {
        return updates;
    };

    for update in updates {
        if let Some(slot) = existing
            .iter_mut()
            .find(|result| result.name == update.name)
        {
            *slot = update;
        } else {
            existing.push(update);
        }
    }

    existing
}

/// Runs the given test paths in a separate thread to avoid stalling the UI,
/// returning the test results when all are complete.
async fn run_tests_in_thread(
    configure: crate::app::ConfigureFn,
    preview_index: usize,
    tests: Vec<PathBuf>,
) -> Vec<TestResult> {
    let handle = tokio::task::spawn_blocking(move || {
        tests
            .into_iter()
            .map(|path| run_test_path(&configure, preview_index, path))
            .collect::<Vec<_>>()
    });

    match handle.await {
        Ok(results) => results,
        Err(error) => vec![TestResult {
            name: "test-runner".to_string(),
            passed: false,
            error: Some(format!("Test runner failed: {error}")),
        }],
    }
}

/// Runs the test at the given `path` and returns the result.
fn run_test_path(
    configure: &crate::app::ConfigureFn,
    preview_index: usize,
    path: PathBuf,
) -> TestResult {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            return TestResult {
                name,
                passed: false,
                error: Some(format!("Failed to read test file: {e}")),
            };
        }
    };

    let ice = match super::Ice::parse(&content) {
        Ok(ice) => ice,
        Err(e) => {
            return TestResult {
                name,
                passed: false,
                error: Some(format!("Failed to parse .ice file: {e}")),
            };
        }
    };

    let mut app = (configure)(crate::App::default());
    if preview_index >= app.descriptors().len() {
        return TestResult {
            name,
            passed: false,
            error: Some("Preview index out of range for test run".to_string()),
        };
    }

    match super::run_single_test(&mut app, preview_index, &ice, &name) {
        Some(error) => TestResult {
            name,
            passed: false,
            error: Some(error),
        },
        None => TestResult {
            name,
            passed: true,
            error: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The run mode of the state indicates whether a test run is currently active.
    #[test]
    fn is_running_reflects_run_mode() {
        let mut state = State::default();
        assert!(!state.is_running());

        state.run_mode = Some(RunMode::All);
        assert!(state.is_running());

        state.run_mode = None;
        assert!(!state.is_running());
    }
}
