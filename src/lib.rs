mod app;
mod message;
mod metadata;
pub mod preview;
mod widget;

use app::App;
pub use message::Message;
pub use metadata::Metadata;
pub use preview::Preview;

// Re-export the attribute macros
pub use snowscape_macros::{stateful, stateless};

pub fn run(configure: fn(App) -> App) -> iced::Result {
    iced::application(move || App::setup(configure), App::update, App::view)
        .title(|app: &App| app.title.clone().unwrap_or("Snowscape Previews".to_owned()))
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
