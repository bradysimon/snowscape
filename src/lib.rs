mod app;
mod message;
mod metadata;
pub mod preview;
mod widget;

use app::App;
pub use message::Message;
pub use metadata::Metadata;
pub use preview::Preview;

// Re-export the attribute macro with a different name to avoid conflict
pub use snowscape_macros::preview;

// Re-export inventory for use in generated code
#[doc(hidden)]
pub use inventory;

use crate::preview::Descriptor;

/// Get all registered previews.
pub fn previews() -> Vec<&'static Descriptor> {
    inventory::iter::<Descriptor>().collect()
}

/// Run the preview application.
pub fn run() -> iced::Result {
    let preview_list = previews();

    if preview_list.is_empty() {
        eprintln!("No previews found. Add #[snowscape::preview] to your functions.");
        return Ok(());
    }

    App::run(preview_list)
}
