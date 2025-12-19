mod app;
mod config_tab;
pub mod icon;
mod message;
pub mod metadata;
pub mod preview;
pub mod style;

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

use app::App;
pub use metadata::Metadata;
use preview::Preview;
pub use preview::{dynamic, stateful, stateless};

pub fn run(configure: fn(App) -> App) -> iced::Result {
    iced::application(move || App::setup(configure), App::update, App::view)
        .title(|app: &App| app.title.clone().unwrap_or("Snowscape Previews".to_owned()))
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
