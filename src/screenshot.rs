//! Screenshot capture support for Snowscape previews.
//!
//! Capture a single preview screenshot without launching the GUI.
//!
//! # Command Line Arguments
//!
//! ```bash
//! cargo run -- --screenshot --preview "Button" --output ./screenshot.png
//! cargo run -- --screenshot --preview "Button"  # saves to ./screenshots/button.png
//! cargo run -- --help  # shows usage
//! ```

use iced::theme::Base;
use iced::{Size, Theme};
use iced_test::Simulator;
use std::path::PathBuf;

/// Parsed screenshot options from CLI args.
#[derive(Debug, Clone)]
pub struct Options {
    /// Name of the preview to capture.
    pub preview: String,
    /// Output path for the PNG file.
    pub output: Option<PathBuf>,
    /// Theme to use for rendering.
    pub theme: Theme,
    /// Viewport size.
    pub viewport_size: Size,
}

/// Result of parsing command-line arguments.
#[derive(Debug)]
pub enum ParseResult {
    /// Run in normal GUI mode.
    RunGui,
    /// Show help message.
    ShowHelp,
    /// Capture a screenshot with the given options.
    Screenshot(Options),
    /// Parse error with message.
    Error(String),
}

/// Parses command-line arguments.
///
/// Not pulling in `clap` to keep dependencies minimal.
pub fn parse_args() -> ParseResult {
    let args: Vec<String> = std::env::args().collect();

    // Check for --help
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return ParseResult::ShowHelp;
    }

    // Check for --screenshot flag
    let screenshot_mode = args.iter().any(|a| a == "--screenshot");
    if !screenshot_mode {
        return ParseResult::RunGui;
    }

    // Parse --preview (required for screenshot mode)
    let Some(preview) = parse_arg(&args, "--preview") else {
        return ParseResult::Error("Screenshot mode requires --preview <name>".to_string());
    };

    // Parse --output (optional)
    let output = parse_arg(&args, "--output").map(PathBuf::from);

    // Parse --theme
    let theme = if let Some(theme_name) = parse_arg(&args, "--theme") {
        // Find theme by name (case-insensitive)
        Theme::ALL
            .iter()
            .find(|t| t.name().to_lowercase() == theme_name.to_lowercase())
            .cloned()
            .unwrap_or_else(|| {
                eprintln!(
                    "Warning: Theme '{}' not found, using Light theme",
                    theme_name
                );
                eprintln!(
                    "Available themes: {}",
                    Theme::ALL
                        .iter()
                        .map(|t| t.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Theme::Light
            })
    } else {
        Theme::Light
    };

    // Parse --size (format: WIDTHxHEIGHT)
    let viewport_size = parse_arg(&args, "--size")
        .and_then(|s| {
            let (w, h) = s.split_once('x')?;
            Some(Size::new(w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or(Size::new(800.0, 600.0));

    ParseResult::Screenshot(Options {
        preview,
        output,
        theme,
        viewport_size,
    })
}

/// Parses a --key value pair from args.
fn parse_arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Returns the help message.
pub fn help_message() -> &'static str {
    r#"Snowscape Preview Runner

USAGE:
    <binary> [OPTIONS]

OPTIONS:
    --screenshot          Capture a screenshot instead of launching GUI
    --preview <name>      Name of the preview to capture (required for screenshot)
    --output <path>       Output path for PNG (default: ./screenshots/<name>.png)
    --theme <name>        Theme for rendering (default: Light)
    --size <WxH>          Viewport size (default: 800x600)
    -h, --help            Show this help message

EXAMPLES:
    # Launch GUI
    cargo run

    # Capture screenshot
    cargo run -- --screenshot --preview "My Button" --output ./button

    # Capture with defaults (saves to ./screenshots/my_button)
    cargo run -- --screenshot --preview "My Button"

    # Capture with dark theme and custom size
    cargo run -- --screenshot --preview "Card" --theme Dark --size 1200x800

    # Capture with Dracula theme
    cargo run -- --screenshot --preview "Card" --theme Dracula
"#
}

/// Error that can occur during screenshot capture.
#[derive(Debug)]
pub enum Error {
    /// Preview not found.
    PreviewNotFound(String),
    /// Failed to create output directory.
    CreateDirectory(std::io::Error),
    /// Failed to save screenshot.
    SaveScreenshot(std::io::Error),
    /// Snapshot error from iced_test.
    Snapshot(iced_test::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PreviewNotFound(name) => write!(f, "Preview not found: '{}'", name),
            Error::CreateDirectory(e) => write!(f, "Failed to create output directory: {}", e),
            Error::SaveScreenshot(e) => write!(f, "Failed to save screenshot: {}", e),
            Error::Snapshot(e) => write!(f, "Snapshot error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<iced_test::Error> for Error {
    fn from(err: iced_test::Error) -> Self {
        Error::Snapshot(err)
    }
}

/// Captures a screenshot of the specified preview.
pub fn capture(app: &crate::App, options: &Options) -> Result<PathBuf, Error> {
    let descriptors = app.descriptors();

    // Find the preview by name (case-insensitive, partial match)
    let preview_index = descriptors
        .iter()
        .position(|d| {
            let label = &d.metadata().label;
            label
                .to_lowercase()
                .contains(&options.preview.to_lowercase())
                || sanitize_name(label) == sanitize_name(&options.preview)
        })
        .ok_or_else(|| Error::PreviewNotFound(options.preview.clone()))?;

    let descriptor = &descriptors[preview_index];
    let label = &descriptor.metadata().label;

    let base_output_path = options.output.clone().unwrap_or_else(|| {
        PathBuf::from("./screenshots").join(format!("{}.png", sanitize_name(label)))
    });

    let parent_dir = base_output_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();

    std::fs::create_dir_all(&parent_dir).map_err(Error::CreateDirectory)?;

    // Find an available filename (increments counter if file already exists)
    let output_path = find_available_path(&base_output_path, &parent_dir)?;

    let mut simulator: Simulator<crate::message::Message> = Simulator::with_size(
        iced::Settings::default(),
        options.viewport_size,
        descriptor.preview.view(),
    );

    let snapshot = simulator.snapshot(&options.theme)?;
    // Creates the screenshot file for us.
    snapshot.matches_image(&output_path)?;

    let actual_path = find_created_file(&output_path, &parent_dir)?;
    Ok(actual_path)
}

/// Finds an available filename by incrementing a counter if needed.
///
/// Returns a path that doesn't conflict with existing files.
/// Note: matches_image will add a renderer suffix (e.g., "-wgpu") to the filename.
fn find_available_path(base_path: &PathBuf, parent_dir: &PathBuf) -> Result<PathBuf, Error> {
    let base_stem = base_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    // Count existing files matching the pattern {base_stem}-*.png (with any renderer suffix)
    let existing_files = std::fs::read_dir(parent_dir)
        .map_err(Error::SaveScreenshot)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let file_name = entry.file_name();
            let file_str = file_name.to_string_lossy();
            file_str.starts_with(&format!("{}-", base_stem)) && file_str.ends_with(".png")
        })
        .count();

    // If files exist, append a counter to make it unique
    if existing_files > 0 {
        Ok(base_path
            .with_file_name(format!("{}-{}", base_stem, existing_files))
            .with_extension("png"))
    } else {
        Ok(base_path.clone())
    }
}

/// Finds the screenshot file created by matches_image.
///
/// matches_image adds a renderer suffix (e.g., "-wgpu", "-tiny-skia") to the filename,
/// so we need to search for the actual file that was created.
fn find_created_file(expected_path: &PathBuf, parent_dir: &PathBuf) -> Result<PathBuf, Error> {
    let final_stem = expected_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    // Find the newest file matching the pattern (it was just created)
    std::fs::read_dir(parent_dir)
        .map_err(Error::SaveScreenshot)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let file_name = entry.file_name();
            let file_str = file_name.to_string_lossy();
            // Match files like: {final_stem}-{renderer}.png
            file_str.starts_with(&format!("{}-", final_stem)) && file_str.ends_with(".png")
        })
        .max_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok())
        .map(|entry| entry.path())
        .ok_or_else(|| {
            Error::SaveScreenshot(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Screenshot file not found after creation (expected pattern: {}-*.png)",
                    final_stem
                ),
            ))
        })
}

/// Sanitizes a name for use as a filename.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}
