//! Visual testing support for Snowscape previews.
//!
//! This module provides functionality to record and run visual tests against
//! previews using Iced's `.ice` test file format.

mod config;
pub mod discovery;
mod error;
pub mod message;
pub mod outcome;
mod session;
mod size_input;
pub mod state;

use iced::keyboard;

pub use config::Config;
pub use discovery::TestInfo;
pub use error::Error;
pub use message::Message;
pub use outcome::Outcome;
pub use session::Session;
pub use state::State;
// Re-export iced_test types for convenience
pub use iced_test::instruction::{Expectation, Interaction, Keyboard, Mouse, Target};
pub use iced_test::{Ice, Instruction};

/// Runs all `.ice` tests in the given directory against previews.
///
/// This function is intended to be called from `#[test]` functions:
///
/// ```ignore
/// #[test]
/// fn visual_tests() -> Result<(), snowscape::test::Error> {
///     snowscape::test::run(my_crate::previews, "tests/")
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
pub fn run<F>(
    configure: F,
    tests_dir: impl AsRef<std::path::Path>,
) -> std::result::Result<(), Error>
where
    F: Fn(crate::App) -> crate::App + Clone,
{
    use std::fs;

    // Build the app with the configure function to get all descriptors
    let initial_app = configure.clone()(crate::App::default());

    let tests_dir = tests_dir.as_ref();
    if !tests_dir.exists() {
        return Err(Error::TestsDirectoryNotFound(tests_dir.to_path_buf()));
    }

    let mut failures = Vec::new();
    let mut test_count = 0;

    // Find all preview folders (directories in tests_dir)
    let preview_folders: Vec<_> = fs::read_dir(tests_dir)
        .map_err(|e| Error::IoError(e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();

    // Also check for legacy flat .ice files in tests_dir
    let legacy_ice_files: Vec<_> = fs::read_dir(tests_dir)
        .map_err(|e| Error::IoError(e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ice"))
        .collect();

    if preview_folders.is_empty() && legacy_ice_files.is_empty() {
        println!(
            "No test folders or .ice files found in {}",
            tests_dir.display()
        );
        return Ok(());
    }

    // Process each preview folder
    for folder_entry in preview_folders {
        let folder_path = folder_entry.path();
        let preview_folder_name = folder_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Find matching preview by sanitized name
        let matching_index = initial_app.descriptors().iter().position(|d| {
            let label = &d.metadata().label;
            discovery::sanitize_name(label) == preview_folder_name
        });

        let Some(preview_index) = matching_index else {
            // No matching preview found for this folder
            println!(
                "Warning: No matching preview found for folder '{}'",
                preview_folder_name
            );
            continue;
        };

        // Find all .ice files in this preview folder
        let ice_files: Vec<_> = match fs::read_dir(&folder_path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "ice"))
                .collect(),
            Err(_) => continue,
        };

        for entry in ice_files {
            let path = entry.path();
            let test_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let full_test_name = format!("{}/{}", preview_folder_name, test_name);
            test_count += 1;

            println!("Running test: {full_test_name}");

            // Load and parse the .ice file
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    failures.push((full_test_name, format!("Failed to read file: {e}")));
                    continue;
                }
            };

            let ice = match Ice::parse(&content) {
                Ok(ice) => ice,
                Err(e) => {
                    failures.push((full_test_name, format!("Failed to parse .ice file: {e}")));
                    continue;
                }
            };

            // Create a fresh app for each test to ensure isolated state
            let mut app = configure.clone()(crate::App::default());

            // Run this test against the preview
            if let Some(error) = run_single_test(&mut app, preview_index, &ice, &full_test_name) {
                failures.push((full_test_name, error));
            } else {
                println!("  ✓ Test passed: {full_test_name}");
            }
        }
    }

    // Process legacy flat .ice files (for backwards compatibility)
    for entry in legacy_ice_files {
        let path = entry.path();
        let test_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        test_count += 1;
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

        // Create a fresh app for the test
        let initial_app = configure.clone()(crate::App::default());

        // Find matching preview by sanitized name (legacy behavior)
        let matching_index = initial_app.descriptors().iter().position(|d| {
            let label = &d.metadata().label;
            discovery::sanitize_name(label) == test_name
        });

        let Some(preview_index) = matching_index else {
            failures.push((
                test_name.to_string(),
                format!("No matching preview found for test '{test_name}'"),
            ));
            continue;
        };

        // Create a fresh app for the test
        let mut app = configure.clone()(crate::App::default());

        if let Some(error) = run_single_test(&mut app, preview_index, &ice, test_name) {
            failures.push((test_name.to_string(), error));
        } else {
            println!("  ✓ Test passed: {test_name}");
        }
    }

    if test_count == 0 {
        println!("No .ice test files found in {}", tests_dir.display());
        return Ok(());
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(Error::TestsFailed(failures))
    }
}

/// Runs a single test against a preview, returning an error message if it fails.
fn run_single_test(
    app: &mut crate::App,
    preview_index: usize,
    ice: &Ice,
    _test_name: &str,
) -> Option<String> {
    use iced_test::Simulator;

    // Create simulator with the preview's initial view
    let mut simulator: Simulator<crate::message::Message> = Simulator::with_size(
        iced::Settings::default(),
        ice.viewport,
        app.descriptors()[preview_index].preview.view(),
    );

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
                if let Err(e) = simulator.find(expected_text.clone()) {
                    return Some(format!(
                        "Expectation failed - text '{}' not found: {}",
                        expected_text, e
                    ));
                }
            }
        }
    }

    None // Test passed
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
