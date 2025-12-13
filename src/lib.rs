mod app;
mod config_tab;
pub mod icon;
mod message;
mod metadata;
pub mod preview;
pub mod style;
mod widget;

use app::App;
use message::Message;
use metadata::Metadata;
use preview::Preview;
pub use preview::{dynamic, stateful, stateless};

pub fn run(configure: fn(App) -> App) -> iced::Result {
    iced::application(move || App::setup(configure), App::update, App::view)
        .title(|app: &App| app.title.clone().unwrap_or("Snowscape Previews".to_owned()))
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
