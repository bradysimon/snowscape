//! Visual testing support for Snowscape previews.
//!
//! This module provides functionality to record and run visual tests against
//! previews using Iced's `.ice` test file format.

use iced::{Size, keyboard};
use std::path::PathBuf;

// Re-export iced_test types for convenience
pub use iced_test::instruction::{Expectation, Interaction, Keyboard, Mouse, Target};
pub use iced_test::{Ice, Instruction};

/// Configuration for a test recording session.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// The size of the test window.
    pub window_size: Size,
    /// The directory where tests are saved.
    pub tests_dir: PathBuf,
    /// Whether to capture a final snapshot for comparison.
    pub capture_snapshot: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            window_size: Size::new(800.0, 600.0),
            tests_dir: PathBuf::from("./tests"),
            capture_snapshot: false,
        }
    }
}

impl TestConfig {
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

/// State for an active test recording session.
#[derive(Debug)]
pub struct TestSession {
    /// The configuration for this session.
    pub config: TestConfig,
    /// The index of the preview being tested.
    pub preview_index: usize,
    /// The name of the preview (used for the test file name).
    pub preview_name: String,
    /// Recorded interactions in Ice format.
    pub instructions: Vec<Instruction>,
    /// Whether recording is currently active.
    pub is_recording: bool,
    /// Current text in the expectation input field.
    pub expect_text_input: String,
    /// Counter for snapshot naming.
    pub snapshot_count: usize,
}

impl TestSession {
    /// Creates a new test session for the given preview.
    pub fn new(config: TestConfig, preview_index: usize, preview_name: String) -> Self {
        Self {
            config,
            preview_index,
            preview_name,
            instructions: Vec::new(),
            is_recording: true,
            expect_text_input: String::new(),
            snapshot_count: 0,
        }
    }

    /// Returns the filename for this test's `.ice` file.
    pub fn ice_filename(&self) -> String {
        // Sanitize the preview name for use as a filename
        let sanitized: String = self
            .preview_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("{}.ice", sanitized.to_lowercase())
    }

    /// Returns the full path where the test file will be saved.
    pub fn ice_path(&self) -> PathBuf {
        self.config.tests_dir.join(self.ice_filename())
    }

    /// Returns the full path where the snapshot will be saved (if enabled).
    pub fn snapshot_path(&self) -> Option<PathBuf> {
        if self.config.capture_snapshot {
            let sanitized: String = self
                .preview_name
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect();
            Some(
                self.config
                    .tests_dir
                    .join(format!("{}.png", sanitized.to_lowercase())),
            )
        } else {
            None
        }
    }

    /// Records an interaction, merging with the previous one if possible.
    pub fn record(&mut self, interaction: Interaction) {
        // Try to merge with the last instruction if it's also an interaction
        if let Some(Instruction::Interact(last)) = self.instructions.pop() {
            let (merged, remainder) = last.merge(interaction);
            self.instructions.push(Instruction::Interact(merged));
            if let Some(r) = remainder {
                self.instructions.push(Instruction::Interact(r));
            }
        } else {
            // No previous instruction or it was an expectation, just add the new one
            self.instructions.push(Instruction::Interact(interaction));
        }
    }

    /// Adds a text expectation to verify the given text is visible.
    pub fn add_text_expectation(&mut self, text: String) {
        if !text.is_empty() {
            self.instructions
                .push(Instruction::Expect(Expectation::Text(text)));
        }
    }

    /// Returns the next snapshot filename for this session.
    pub fn next_snapshot_name(&mut self) -> String {
        let sanitized: String = self
            .preview_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        self.snapshot_count += 1;
        format!("{}_{}.png", sanitized.to_lowercase(), self.snapshot_count)
    }

    /// Converts the session to an Ice structure for serialization.
    pub fn to_ice(&self) -> Ice {
        Ice {
            viewport: self.config.window_size,
            mode: iced_test::emulator::Mode::Immediate,
            preset: None,
            instructions: self.instructions.clone(),
        }
    }

    /// Saves the test to disk using the .ice format.
    pub fn save(&self) -> std::io::Result<()> {
        // Ensure the tests directory exists
        std::fs::create_dir_all(&self.config.tests_dir)?;

        // Convert to Ice and write
        let ice = self.to_ice();
        std::fs::write(self.ice_path(), ice.to_string())?;

        Ok(())
    }

    /// Returns the sanitized name used for filenames.
    pub fn sanitized_name(&self) -> String {
        self.preview_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }
}

/// Runs all `.ice` tests in the given directory against previews.
///
/// This function is intended to be called from `#[test]` functions:
///
/// ```ignore
/// #[test]
/// fn visual_tests() -> Result<(), snowscape::test::Error> {
///     snowscape::test::run(my_crate::configure, "tests/")
/// }
/// ```
///
/// The `configure` function should be the same one used with `snowscape::run`,
/// allowing test and application code to share the same preview definitions.
///
/// # Arguments
///
/// * `configure` - A function that configures the App with previews
/// * `tests_dir` - The directory containing `.ice` test files
///
/// # Returns
///
/// Returns `Ok(())` if all tests pass, or an error describing failures.
pub fn run<F>(configure: F, tests_dir: impl AsRef<std::path::Path>) -> Result<(), Error>
where
    F: Fn(crate::App) -> crate::App,
{
    use iced_test::Simulator;
    use std::fs;

    // Build the app with the configure function to get all descriptors
    let mut app = configure(crate::App::default());

    let tests_dir = tests_dir.as_ref();
    if !tests_dir.exists() {
        return Err(Error::TestsDirectoryNotFound(tests_dir.to_path_buf()));
    }

    let mut failures = Vec::new();

    // Find all .ice files
    let ice_files: Vec<_> = fs::read_dir(tests_dir)
        .map_err(|e| Error::IoError(e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ice"))
        .collect();

    if ice_files.is_empty() {
        println!("No .ice test files found in {}", tests_dir.display());
        return Ok(());
    }

    for entry in ice_files {
        let path = entry.path();
        let test_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!("Running test: {test_name}");

        // Load and parse the .ice file
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                failures.push((test_name.to_string(), format!("Failed to read file: {e}")));
                continue;
            }
        };

        let ice = match Ice::parse(&content) {
            Ok(ice) => ice,
            Err(e) => {
                failures.push((
                    test_name.to_string(),
                    format!("Failed to parse .ice file: {e}"),
                ));
                continue;
            }
        };

        // Find matching preview by sanitized name
        let matching_index = app.descriptors().iter().position(|d| {
            let label = &d.metadata().label;
            let sanitized: String = label
                .chars()
                .map(|c: char| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>()
                .to_lowercase();
            sanitized == test_name
        });

        let Some(preview_index) = matching_index else {
            failures.push((
                test_name.to_string(),
                format!("No matching preview found for test '{test_name}'"),
            ));
            continue;
        };

        // Create simulator with the preview's initial view
        let mut simulator: Simulator<crate::message::Message> = Simulator::with_size(
            iced::Settings::default(),
            ice.viewport,
            app.descriptors()[preview_index].preview.view(),
        );

        // Track if this test had any failures
        let failures_before = failures.len();

        // Run each instruction
        for instruction in &ice.instructions {
            match instruction {
                Instruction::Interact(interaction) => {
                    let events = interaction.events(|target| match target {
                        Target::Point(p) => Some(*p),
                        Target::Text(_) => None,
                    });

                    match events {
                        Some(event_list) => {
                            simulator.simulate(event_list);
                        }
                        None => {
                            match interaction {
                                Interaction::Mouse(Mouse::Click { target, .. }) => {
                                    if let Some(Target::Text(text)) = target {
                                        let _ = simulator.click(text.as_str());
                                    }
                                }
                                Interaction::Keyboard(Keyboard::Typewrite(text)) => {
                                    simulator.typewrite(text);
                                }
                                _ => {
                                    // Fallback to event conversion for other cases
                                    let events = interaction_to_events(interaction);
                                    simulator.simulate(events);
                                }
                            }
                        }
                    }

                    // Get messages produced by the interaction and update the preview
                    for message in simulator.into_messages() {
                        let _ = app.descriptors_mut()[preview_index].preview.update(message);
                    }

                    // Regenerate the simulator with the updated view
                    simulator = Simulator::with_size(
                        iced::Settings::default(),
                        ice.viewport,
                        app.descriptors()[preview_index].preview.view(),
                    );
                }
                Instruction::Expect(Expectation::Text(expected_text)) => {
                    // Try to find the expected text in the UI
                    match simulator.find(expected_text.clone()) {
                        Ok(_) => {} // Text found
                        Err(e) => {
                            failures.push((
                                test_name.to_string(),
                                format!(
                                    "Expectation failed - text '{expected_text}' not found: {e}",
                                ),
                            ));
                        }
                    }
                }
            }
        }

        // Only print success if no failures were added for this test
        if failures.len() == failures_before {
            println!("  ✓ Test passed: {test_name}");
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::TestsFailed(failures))
    }
}

/// Converts an Interaction to a sequence of iced Events.
fn interaction_to_events(interaction: &Interaction) -> Vec<iced::Event> {
    use iced::{Event, mouse};

    match interaction {
        Interaction::Mouse(mouse_action) => match mouse_action {
            Mouse::Move(target) => {
                let position = target_to_point(target);
                vec![Event::Mouse(mouse::Event::CursorMoved { position })]
            }
            Mouse::Press { button, target } => {
                let mut events = Vec::new();
                if let Some(t) = target {
                    events.push(Event::Mouse(mouse::Event::CursorMoved {
                        position: target_to_point(t),
                    }));
                }
                events.push(Event::Mouse(mouse::Event::ButtonPressed(*button)));
                events
            }
            Mouse::Release { button, target } => {
                let mut events = Vec::new();
                if let Some(t) = target {
                    events.push(Event::Mouse(mouse::Event::CursorMoved {
                        position: target_to_point(t),
                    }));
                }
                events.push(Event::Mouse(mouse::Event::ButtonReleased(*button)));
                events
            }
            Mouse::Click { button, target } => {
                let mut events = Vec::new();
                if let Some(t) = target {
                    events.push(Event::Mouse(mouse::Event::CursorMoved {
                        position: target_to_point(t),
                    }));
                }
                events.push(Event::Mouse(mouse::Event::ButtonPressed(*button)));
                events.push(Event::Mouse(mouse::Event::ButtonReleased(*button)));
                events
            }
        },
        Interaction::Keyboard(keyboard_action) => match keyboard_action {
            Keyboard::Typewrite(text) => {
                // Each character becomes a key press with text
                text.chars()
                    .flat_map(|c| {
                        let key = keyboard::Key::Character(c.to_string().into());
                        vec![Event::Keyboard(keyboard::Event::KeyPressed {
                            key: key.clone(),
                            modified_key: key,
                            physical_key: keyboard::key::Physical::Unidentified(
                                keyboard::key::NativeCode::Unidentified,
                            ),
                            location: keyboard::Location::Standard,
                            modifiers: keyboard::Modifiers::empty(),
                            text: Some(c.to_string().into()),
                            repeat: false,
                        })]
                    })
                    .collect()
            }
            Keyboard::Press(key) => {
                let iced_key = special_key_to_iced(key);
                vec![Event::Keyboard(keyboard::Event::KeyPressed {
                    key: iced_key.clone(),
                    modified_key: iced_key,
                    physical_key: keyboard::key::Physical::Unidentified(
                        keyboard::key::NativeCode::Unidentified,
                    ),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::empty(),
                    text: None,
                    repeat: false,
                })]
            }
            Keyboard::Release(key) => {
                let iced_key = special_key_to_iced(key);
                vec![Event::Keyboard(keyboard::Event::KeyReleased {
                    key: iced_key.clone(),
                    modified_key: iced_key,
                    physical_key: keyboard::key::Physical::Unidentified(
                        keyboard::key::NativeCode::Unidentified,
                    ),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::empty(),
                })]
            }
            Keyboard::Type(key) => {
                // A key was "typed" (press and release)
                let iced_key = special_key_to_iced(key);
                vec![
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: iced_key.clone(),
                        modified_key: iced_key.clone(),
                        physical_key: keyboard::key::Physical::Unidentified(
                            keyboard::key::NativeCode::Unidentified,
                        ),
                        location: keyboard::Location::Standard,
                        modifiers: keyboard::Modifiers::empty(),
                        text: None,
                        repeat: false,
                    }),
                    Event::Keyboard(keyboard::Event::KeyReleased {
                        key: iced_key.clone(),
                        modified_key: iced_key,
                        physical_key: keyboard::key::Physical::Unidentified(
                            keyboard::key::NativeCode::Unidentified,
                        ),
                        location: keyboard::Location::Standard,
                        modifiers: keyboard::Modifiers::empty(),
                    }),
                ]
            }
        },
    }
}

/// Converts a Target to a Point.
fn target_to_point(target: &Target) -> iced::Point {
    match target {
        Target::Point(p) => *p,
        Target::Text(_) => {
            // For text targets, we'd need to look up the element's position
            // For now, default to origin - the Simulator.find() handles text targets
            iced::Point::ORIGIN
        }
    }
}

/// Converts a special key to an iced Key.
fn special_key_to_iced(key: &iced_test::instruction::Key) -> keyboard::Key {
    use iced_test::instruction::Key;
    match key {
        Key::Enter => keyboard::Key::Named(keyboard::key::Named::Enter),
        Key::Escape => keyboard::Key::Named(keyboard::key::Named::Escape),
        Key::Tab => keyboard::Key::Named(keyboard::key::Named::Tab),
        Key::Backspace => keyboard::Key::Named(keyboard::key::Named::Backspace),
    }
}

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
