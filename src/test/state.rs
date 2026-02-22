//! Test state management encapsulating all test-related functionality.

use std::path::PathBuf;

use iced::{Task, window};

use crate::test;

use super::{
    Config, Session,
    discovery::{TestInfo, delete_test, discover_tests, sanitize_name},
    message::Message,
    size_input::SizeInput,
};

/// Encapsulates all test-related state and logic.
#[derive(Debug)]
pub struct State {
    /// Test configuration for the test window.
    pub config: Config,
    /// The width input for the test window.
    pub width_input: SizeInput,
    /// The height input for the test window.
    pub height_input: SizeInput,
    /// The test name input for naming new tests.
    pub name_input: String,
    /// The active test recording session, if any.
    pub session: Option<Session>,
    /// The ID of the test window when recording.
    pub window_id: Option<window::Id>,
    /// Discovered tests for the current preview.
    pub discovered_tests: Vec<TestInfo>,
    /// Results from the last test run.
    pub last_run_results: Option<Vec<test::Outcome>>,
    /// The scope of the current test run, if any.
    pub run_mode: Option<RunMode>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            config: Config::default(),
            width_input: SizeInput::new("800"),
            height_input: SizeInput::new("600"),
            name_input: String::new(),
            session: None,
            window_id: None,
            discovered_tests: Vec::new(),
            last_run_results: None,
            run_mode: None,
        }
    }
}

/// The mode of an active test run, indicating which tests are being executed.
#[derive(Debug, Clone)]
pub enum RunMode {
    /// All tests for the current preview are being run.
    All,
    /// A single test at the given path is being run.
    Single(PathBuf),
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
        self.width_input.is_valid()
            && self.height_input.is_valid()
            && !self.name_input.trim().is_empty()
            && !self.is_existing_test(self.test_name())
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
    fn test_name(&self) -> &str {
        self.name_input.trim()
    }

    /// Updates the test state based on the given message.
    ///
    /// Returns a task that may need to be executed.
    pub fn update(&mut self, message: Message, ctx: Option<UpdateContext<'_>>) -> Task<Message> {
        match message {
            Message::ChangeWidth(width) => {
                self.width_input.update(width);
                if let Some(w) = self.width_input.value() {
                    self.config.window_size.width = w;
                }
                Task::none()
            }
            Message::ChangeHeight(height) => {
                self.height_input.update(height);
                if let Some(h) = self.height_input.value() {
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
                    test_name.to_string(),
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

                capture_snapshot_for_session(session, ctx);

                // Close the test window
                if let Some(test_window_id) = self.window_id.take() {
                    window::close(test_window_id).chain(Task::done(Message::RemoveSession))
                } else {
                    Task::none()
                }
            }
            Message::ChangeExpectText(text) => {
                if let Some(session) = &mut self.session {
                    session.expect_text_input = text;
                }
                Task::none()
            }
            Message::AddTextExpectation => {
                if let Some(session) = &mut self.session {
                    let text = session.expect_text_input.trim();
                    if text.is_empty() {
                        return Task::none();
                    }
                    let text = text.to_string();
                    session.expect_text_input.clear();
                    session.add_text_expectation(text);
                }
                Task::none()
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

                self.run_mode = Some(RunMode::Single(path.clone()));

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

                let test_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                self.discovered_tests = discover_tests(&self.config.tests_dir, &preview_name);
                self.remove_test_result(&test_name);
                Task::none()
            }
            Message::RunComplete(results) => {
                self.last_run_results = Some(match self.run_mode {
                    Some(RunMode::Single(_)) => {
                        merge_run_results(self.last_run_results.take(), results)
                    }
                    _ => results,
                });
                self.run_mode = None;
                Task::none()
            }
        }
    }

    /// Removes the test result with the given `test_name` from the last run results, if present.
    fn remove_test_result(&mut self, test_name: &str) {
        if let Some(results) = self.last_run_results.as_mut() {
            if let Some(index) = results.iter().position(|item| item.name == test_name) {
                results.remove(index);
            }
        }
    }

    /// Whether the given `test_name` already exists in the discovered tests.
    fn is_existing_test(&self, test_name: &str) -> bool {
        let sanitized_name = sanitize_name(test_name);

        self.discovered_tests
            .iter()
            .any(|test| sanitize_name(&test.name) == sanitized_name)
    }
}

/// Captures a baseline snapshot for the completed recording session, when enabled.
fn capture_snapshot_for_session(session: &Session, ctx: Option<UpdateContext<'_>>) {
    if !session.config.capture_snapshot {
        return;
    }

    let Some(ctx) = ctx else {
        eprintln!("Failed to capture snapshot: missing update context");
        return;
    };

    let Some(configure) = ctx.configure.clone() else {
        eprintln!("Failed to capture snapshot: missing configure callback");
        return;
    };

    let mut app = (configure)(crate::App::default());
    let ice = session.to_ice();

    if let Some(snapshot_path) = session.snapshot_path() {
        if let Err(e) = super::capture_baseline_screenshot(
            &mut app,
            session.preview_index,
            &ice,
            &snapshot_path,
        ) {
            eprintln!("Failed to capture snapshot: {}", e);
        }
    }
}

fn merge_run_results(
    previous: Option<Vec<test::Outcome>>,
    updates: Vec<test::Outcome>,
) -> Vec<test::Outcome> {
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
) -> Vec<test::Outcome> {
    let mut tasks = tokio::task::JoinSet::new();

    for path in tests {
        let configure = configure.clone();
        tasks.spawn(async move { run_test_path(configure, preview_index, path).await });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(outcome) => results.push(outcome),
            Err(error) => results.push(test::Outcome::failed(
                "test-runner",
                format!("[Internal error] Test runner failed: {error}"),
            )),
        }
    }

    results
}

/// Runs the test at the given `path` and returns the result.
async fn run_test_path(
    configure: crate::app::ConfigureFn,
    preview_index: usize,
    path: PathBuf,
) -> test::Outcome {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let content = match tokio::fs::read_to_string(&path).await {
        Ok(content) => content,
        Err(e) => {
            return test::Outcome::failed(name, format!("Failed to read test file: {e}"));
        }
    };

    let ice = match super::Ice::parse(&content) {
        Ok(ice) => ice,
        Err(e) => {
            return test::Outcome::failed(name, format!("Failed to parse .ice file: {e}"));
        }
    };

    let mut app = (configure)(crate::App::default());
    if preview_index >= app.descriptors().len() {
        return test::Outcome::failed(name, "Preview index out of range for test run");
    }

    match super::run_single_test(&mut app, preview_index, &ice, &path) {
        Some(error) => test::Outcome::failed(name, error),
        None => test::Outcome::passed(name),
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

    /// The record button requires a non-empty name and valid size inputs.
    #[test]
    fn can_record_requires_valid_sizes() {
        let mut state = State::default();
        state.name_input = "test-name".to_string();

        state.width_input.update("".to_string());
        assert!(!state.can_record());
    }

    /// The record button accepts positive size inputs.
    #[test]
    fn can_record_accepts_positive_sizes() {
        let mut state = State::default();
        state.name_input = "test-name".to_string();
        state.width_input.update("800".to_string());
        state.height_input.update("600".to_string());

        assert!(state.can_record());
    }

    /// The record button remains disabled when the name is empty.
    #[test]
    fn can_record_requires_name() {
        let mut state = State::default();
        state.name_input = "".to_string();
        state.width_input.update("800".to_string());
        state.height_input.update("600".to_string());

        assert!(!state.can_record());
    }

    /// The record button stays disabled when the entered name conflicts after sanitization.
    #[test]
    fn can_record_rejects_sanitized_name_conflict() {
        let mut state = State::default();
        state.name_input = "some test".to_string();
        state.width_input.update("800".to_string());
        state.height_input.update("600".to_string());
        state.discovered_tests = vec![TestInfo {
            name: "some-test".to_string(),
            path: PathBuf::from("./tests/counter/some-test.ice"),
            preview: "counter".to_string(),
            has_snapshot: false,
        }];

        assert!(!state.can_record());
    }

    /// Existing test detection compares names by their sanitized form.
    #[test]
    fn is_existing_test_uses_sanitized_names() {
        let mut state = State::default();
        state.discovered_tests = vec![TestInfo {
            name: "some-test".to_string(),
            path: PathBuf::from("./tests/counter/some-test.ice"),
            preview: "counter".to_string(),
            has_snapshot: false,
        }];

        assert!(state.is_existing_test("some test"));
        assert!(state.is_existing_test("  SOME TEST  "));
        assert!(!state.is_existing_test("different test"));
    }

    /// Existing test detection also handles hyphens and spaces consistently.
    #[test]
    fn is_existing_test_matches_hyphen_and_space_variants() {
        let mut state = State::default();
        state.discovered_tests = vec![TestInfo {
            name: "some-test".to_string(),
            path: PathBuf::from("./tests/counter/some-test.ice"),
            preview: "counter".to_string(),
            has_snapshot: false,
        }];

        assert!(state.is_existing_test("some-test"));
        assert!(state.is_existing_test("some test"));
    }

    /// We should be able to remove test results by name.
    #[test]
    fn remove_test_result() {
        let mut state = State::default();
        state.last_run_results = Some(vec![
            test::Outcome::passed("test1"),
            test::Outcome::failed("test2", "Error message"),
        ]);

        state.remove_test_result("test1");
        assert_eq!(
            state.last_run_results,
            Some(vec![test::Outcome::failed("test2", "Error message")])
        );

        state.remove_test_result("nonexistent");
        assert_eq!(
            state.last_run_results,
            Some(vec![test::Outcome::failed("test2", "Error message")])
        );

        state.remove_test_result("test2");
        assert_eq!(state.last_run_results, Some(vec![]));
    }
}
