mod app;
mod config_tab;
pub mod icon;
mod message;
pub mod metadata;
pub mod preview;
pub mod screenshot;
pub mod style;
pub mod test;

#[cfg(feature = "internal")]
pub mod widget;
#[cfg(not(feature = "internal"))]
mod widget;
#[cfg(feature = "internal")]
pub use crate::config_tab::ConfigTab;

#[cfg(not(feature = "internal"))]
use message::Message;
#[cfg(feature = "internal")]
pub use message::Message;

pub use app::App;

pub use metadata::Metadata;
use preview::Preview;
pub use preview::{dynamic, stateful, stateless};

/// Runs the Snowscape preview application.
///
/// Runs the application previews by default, and supports passing command-line arguments
/// for capturing screenshots of a specific preview.
///
/// ```bash
/// cargo run -- --screenshot --preview "Button" --output ./screenshot.png
/// ```
pub fn run<F>(configure: F) -> iced::Result
where
    F: Fn(App) -> App + 'static,
{
    match screenshot::parse_args() {
        screenshot::ParseResult::ShowHelp => {
            println!("{}", screenshot::help_message());
            Ok(())
        }
        screenshot::ParseResult::Error(msg) => {
            eprintln!("Error: {msg}\n");
            eprintln!("Run with --help for usage information.");
            std::process::exit(1);
        }
        screenshot::ParseResult::Screenshot(options) => {
            let app = configure(App::default());
            match screenshot::capture(&app, &options) {
                Ok(path) => {
                    println!("Screenshot saved: {}", path.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to capture screenshot: {e}");
                    std::process::exit(1);
                }
            }
        }
        screenshot::ParseResult::RunGui => {
            iced::daemon(move || App::setup(&configure), App::update, App::view)
                .title(App::window_title)
                .theme(App::theme)
                .subscription(App::subscription)
                .run()
        }
    }
}
