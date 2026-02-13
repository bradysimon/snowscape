use std::path::PathBuf;

use iced_test::{
    Ice, Instruction,
    instruction::{Expectation, Interaction},
};

use crate::test::{Config, discovery::sanitize_name};

/// State for an active test recording session.
#[derive(Debug)]
pub struct Session {
    /// The configuration for this session.
    pub config: Config,
    /// The index of the preview being tested.
    pub preview_index: usize,
    /// The name of the preview (used for folder organization).
    pub preview_name: String,
    /// The name of the test (used for the `.ice` filename).
    pub test_name: String,
    /// Recorded interactions in Ice format.
    pub instructions: Vec<Instruction>,
    /// Whether recording is currently active.
    pub is_recording: bool,
    /// Current text in the expectation input field.
    pub expect_text_input: String,
    /// Counter for snapshot naming.
    pub snapshot_count: usize,
}

impl Session {
    /// Creates a new test session for the given preview.
    pub fn new(
        config: Config,
        preview_index: usize,
        preview_name: String,
        test_name: String,
    ) -> Self {
        Self {
            config,
            preview_index,
            preview_name,
            test_name,
            instructions: Vec::new(),
            is_recording: true,
            expect_text_input: String::new(),
            snapshot_count: 0,
        }
    }

    /// Returns the sanitized preview name for folder naming.
    pub fn sanitized_preview_name(&self) -> String {
        sanitize_name(&self.preview_name)
    }

    /// Returns the sanitized test name for file naming.
    pub fn sanitized_test_name(&self) -> String {
        sanitize_name(&self.test_name)
    }

    /// Returns the filename for this test's `.ice` file.
    pub fn ice_filename(&self) -> String {
        format!("{}.ice", self.sanitized_test_name())
    }

    /// Returns the directory for this preview's tests.
    pub fn preview_dir(&self) -> PathBuf {
        self.config.tests_dir.join(self.sanitized_preview_name())
    }

    /// Returns the full path where the test file will be saved.
    pub fn ice_path(&self) -> PathBuf {
        self.preview_dir().join(self.ice_filename())
    }

    /// Returns the full path where the snapshot will be saved (if enabled).
    pub fn snapshot_path(&self) -> Option<PathBuf> {
        if self.config.capture_snapshot {
            Some(
                self.preview_dir()
                    .join(format!("{}.png", self.sanitized_test_name())),
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
        self.snapshot_count += 1;
        format!("{}_{}.png", self.sanitized_test_name(), self.snapshot_count)
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
        // Ensure the preview's tests directory exists
        std::fs::create_dir_all(self.preview_dir())?;

        // Convert to Ice and write
        let ice = self.to_ice();
        std::fs::write(self.ice_path(), ice.to_string())?;

        Ok(())
    }
}
